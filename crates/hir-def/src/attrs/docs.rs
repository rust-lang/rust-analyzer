//! Documentation extraction and source mapping.
//!
//! This module handles the extraction and processing of doc comments and `#[doc = "..."]`
//! attributes, including macro expansion for `#[doc = macro!()]` patterns.
//! It builds a concatenated string of the full docs as well as a source map
//! to map it back to AST (which is needed for things like resolving links in doc comments
//! and highlight injection).

use std::{
    convert::Infallible,
    ops::{ControlFlow, Range},
};

use base_db::{Crate, SourceDatabase};
use cfg::CfgOptions;
use either::Either;
use hir_expand::{
    AstId, ExpandTo, HirFileId, InFile,
    attrs::{AstPathExt, expand_cfg_attr_with_doc_comments},
    mod_path::ModPath,
    span_map::SpanMap,
};
use span::AstIdMap;
use syntax::{
    AstNode, AstToken, SyntaxNode,
    ast::{self, AttrDocCommentIter, IsString},
};
use tt::{TextRange, TextSize};

use crate::{macro_call_as_call_id, nameres::MacroSubNs, resolver::Resolver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DocsSourceMapLine {
    /// The offset in [`Docs::docs`].
    string_offset: TextSize,
    /// The offset in the AST of the text. `None` for macro-expanded doc strings
    /// where we cannot provide a faithful source mapping.
    ast_offset: Option<TextSize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Docs {
    /// The concatenated string of all `#[doc = "..."]` attributes and documentation comments.
    docs: String,
    /// A sorted map from an offset in `docs` to an offset in the source code.
    docs_source_map: Vec<DocsSourceMapLine>,
    /// If the item is an outlined module (`mod foo;`), `docs_source_map` store the concatenated
    /// list of the outline and inline docs (outline first). Then, this field contains the [`HirFileId`]
    /// of the outline declaration, and the index in `docs` from which the inline docs
    /// begin.
    outline_mod: Option<(HirFileId, usize)>,
    inline_file: HirFileId,
    /// The size the prepended prefix, which does not map to real doc comments.
    prefix_len: TextSize,
    /// The offset in `docs` from which the docs are inner attributes/comments.
    inline_inner_docs_start: Option<TextSize>,
    /// Like `inline_inner_docs_start`, but for `outline_mod`. This can happen only when merging `Docs`
    /// (as outline modules don't have inner attributes).
    outline_inner_docs_start: Option<TextSize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsInnerDoc {
    No,
    Yes,
}

impl IsInnerDoc {
    #[inline]
    pub fn yes(self) -> bool {
        self == IsInnerDoc::Yes
    }
}

impl Docs {
    #[inline]
    pub fn docs(&self) -> &str {
        &self.docs
    }

    #[inline]
    pub fn into_docs(self) -> String {
        self.docs
    }

    pub fn find_ast_range(
        &self,
        mut string_range: TextRange,
    ) -> Option<(InFile<TextRange>, IsInnerDoc)> {
        if string_range.start() < self.prefix_len {
            return None;
        }
        string_range -= self.prefix_len;

        let mut file = self.inline_file;
        let mut inner_docs_start = self.inline_inner_docs_start;
        // Check whether the range is from the outline, the inline, or both.
        let source_map = if let Some((outline_mod_file, outline_mod_end)) = self.outline_mod {
            if let Some(first_inline) = self.docs_source_map.get(outline_mod_end) {
                if string_range.end() <= first_inline.string_offset {
                    // The range is completely in the outline.
                    file = outline_mod_file;
                    inner_docs_start = self.outline_inner_docs_start;
                    &self.docs_source_map[..outline_mod_end]
                } else if string_range.start() >= first_inline.string_offset {
                    // The range is completely in the inline.
                    &self.docs_source_map[outline_mod_end..]
                } else {
                    // The range is combined from the outline and the inline - cannot map it back.
                    return None;
                }
            } else {
                // There is no inline.
                file = outline_mod_file;
                inner_docs_start = self.outline_inner_docs_start;
                &self.docs_source_map
            }
        } else {
            // There is no outline.
            &self.docs_source_map
        };

        let after_range =
            source_map.partition_point(|line| line.string_offset <= string_range.start()) - 1;
        let after_range = &source_map[after_range..];
        let line = after_range.first()?;
        // Unmapped lines (from macro-expanded docs) cannot be mapped back to AST.
        let ast_offset = line.ast_offset?;
        if after_range.get(1).is_some_and(|next_line| next_line.string_offset < string_range.end())
        {
            // The range is combined from two lines - cannot map it back.
            return None;
        }
        let ast_range = string_range - line.string_offset + ast_offset;
        let is_inner = if inner_docs_start
            .is_some_and(|inner_docs_start| string_range.start() >= inner_docs_start)
        {
            IsInnerDoc::Yes
        } else {
            IsInnerDoc::No
        };
        Some((InFile::new(file, ast_range), is_inner))
    }

    #[inline]
    pub fn shift_by(&mut self, offset: TextSize) {
        self.prefix_len += offset;
    }

    pub fn prepend_str(&mut self, s: &str) {
        self.prefix_len += TextSize::of(s);
        self.docs.insert_str(0, s);
    }

    pub fn append_str(&mut self, s: &str) {
        self.docs.push_str(s);
    }

    pub fn append(&mut self, other: &Docs) {
        let other_offset = TextSize::of(&self.docs);

        assert!(
            self.outline_mod.is_none() && other.outline_mod.is_none(),
            "cannot merge `Docs` that have `outline_mod` set"
        );
        self.outline_mod = Some((self.inline_file, self.docs_source_map.len()));
        self.inline_file = other.inline_file;
        self.outline_inner_docs_start = self.inline_inner_docs_start;
        self.inline_inner_docs_start = other.inline_inner_docs_start.map(|it| it + other_offset);

        self.docs.push_str(&other.docs);
        self.docs_source_map.extend(other.docs_source_map.iter().map(
            |&DocsSourceMapLine { string_offset, ast_offset }| DocsSourceMapLine {
                ast_offset,
                string_offset: string_offset + other_offset,
            },
        ));
    }

    fn extend_with_doc_comment(&mut self, comment: ast::Comment, indent: &mut usize) {
        let is_block = comment.kind().shape == ast::CommentShape::Block;
        let Some((doc, offset)) = comment.doc_comment() else { return };
        self.extend_with_doc_str(
            doc,
            comment.syntax().text_range().start() + offset,
            indent,
            is_block,
        );
    }

    fn extend_with_doc_attr(&mut self, value: ast::String, indent: &mut usize) {
        let Some(value_offset) = value.text_range_between_quotes() else { return };
        let value_offset = value_offset.start();
        let Ok(value) = value.value() else { return };
        // FIXME: Handle source maps for escaped text.
        self.extend_with_doc_str(&value, value_offset, indent, false);
    }

    pub(crate) fn extend_with_doc_str(
        &mut self,
        doc: &str,
        offset_in_ast: TextSize,
        indent: &mut usize,
        is_block: bool,
    ) {
        self.push_doc_lines(doc, Some(offset_in_ast), indent, is_block);
    }

    fn extend_with_unmapped_doc_str(&mut self, doc: &str, indent: &mut usize) {
        self.push_doc_lines(doc, None, indent, false);
    }

    fn push_doc_lines(
        &mut self,
        doc: &str,
        mut ast_offset: Option<TextSize>,
        indent: &mut usize,
        is_block: bool,
    ) {
        let lines = doc.split('\n').collect::<Vec<_>>();
        // For block doc comments (`/** ... */`), strip the common leading `*` decoration, mirroring
        // rustdoc's `beautify_doc_string`. This is `None` when the block has no consistent star column
        // (or for line comments and `#[doc = "..."]` attributes), leaving the text untouched.
        let star_trim = if is_block { block_star_horizontal_trim(&lines) } else { None };

        for &raw_line in &lines {
            // Offsets are tracked against the original line. A trailing `\r` (CRLF) is ASCII at the end,
            // so ignoring it here does not affect the leading-byte accounting below.
            let orig_len = TextSize::of(raw_line);
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            let (line, stripped_len) = match star_trim {
                Some(prefix) => strip_block_star_prefix(line, prefix),
                None => (line, TextSize::new(0)),
            };

            self.docs_source_map.push(DocsSourceMapLine {
                string_offset: TextSize::of(&self.docs),
                // The removed prefix is ASCII, so the AST offset advances by exactly the stripped bytes.
                ast_offset: ast_offset.map(|it| it + stripped_len),
            });
            if let Some(ref mut offset) = ast_offset {
                *offset += orig_len + TextSize::of("\n");
            }

            let line = line.trim_end();
            if let Some(line_indent) = line.chars().position(|ch| !ch.is_whitespace()) {
                // Empty lines are handled because `position()` returns `None` for them.
                *indent = std::cmp::min(*indent, line_indent);
            }
            self.docs.push_str(line);
            self.docs.push('\n');
        }
    }

    fn remove_indent(&mut self, indent: usize, start_source_map_index: usize) {
        /// In case of panics, we want to avoid corrupted UTF-8 in `self.docs`, so we clear it.
        struct Guard<'a>(&'a mut Docs);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                let Docs {
                    docs,
                    docs_source_map,
                    outline_mod,
                    inline_file: _,
                    prefix_len: _,
                    inline_inner_docs_start: _,
                    outline_inner_docs_start: _,
                } = self.0;
                // Don't use `String::clear()` here because it's not guaranteed to not do UTF-8-dependent things,
                // and we may have temporarily broken the string's encoding.
                unsafe { docs.as_mut_vec() }.clear();
                // This is just to avoid panics down the road.
                docs_source_map.clear();
                *outline_mod = None;
            }
        }

        if self.docs.is_empty() {
            return;
        }

        let guard = Guard(self);
        let source_map = &mut guard.0.docs_source_map[start_source_map_index..];
        let Some(&DocsSourceMapLine { string_offset: mut copy_into, .. }) = source_map.first()
        else {
            return;
        };
        // We basically want to remove multiple ranges from a string. Doing this efficiently (without O(N^2)
        // or allocations) requires unsafe. Basically, for each line, we copy the line minus the indent into
        // consecutive to the previous line (which may have moved). Then at the end we truncate.
        let mut accumulated_offset = TextSize::new(0);
        for idx in 0..source_map.len() {
            let string_end_offset = source_map
                .get(idx + 1)
                .map_or_else(|| TextSize::of(&guard.0.docs), |next_attr| next_attr.string_offset);
            let line_source = &mut source_map[idx];
            let line_docs =
                &guard.0.docs[TextRange::new(line_source.string_offset, string_end_offset)];
            let line_docs_len = TextSize::of(line_docs);
            let indent_size = line_docs.char_indices().nth(indent).map_or_else(
                || TextSize::of(line_docs) - TextSize::of("\n"),
                |(offset, _)| TextSize::new(offset as u32),
            );
            unsafe { guard.0.docs.as_bytes_mut() }.copy_within(
                Range::<usize>::from(TextRange::new(
                    line_source.string_offset + indent_size,
                    string_end_offset,
                )),
                copy_into.into(),
            );
            copy_into += line_docs_len - indent_size;

            if let Some(inner_attrs_start) = &mut guard.0.inline_inner_docs_start
                && *inner_attrs_start == line_source.string_offset
            {
                *inner_attrs_start -= accumulated_offset;
            }
            // The removals in the string accumulate, but in the AST not, because it already points
            // to the beginning of each attribute.
            // Also, we need to shift the AST offset of every line, but the string offset of the first
            // line should not get shifted (in general, the shift for the string offset is by the
            // number of lines until the current one, excluding the current one).
            line_source.string_offset -= accumulated_offset;
            if let Some(ref mut ast_offset) = line_source.ast_offset {
                *ast_offset += indent_size;
            }

            accumulated_offset += indent_size;
        }
        // Don't use `String::truncate()` here because it's not guaranteed to not do UTF-8-dependent things,
        // and we may have temporarily broken the string's encoding.
        unsafe { guard.0.docs.as_mut_vec() }.truncate(copy_into.into());

        std::mem::forget(guard);
    }

    fn remove_last_newline(&mut self) {
        self.docs.truncate(self.docs.len().saturating_sub(1));
    }

    fn shrink_to_fit(&mut self) {
        let Docs {
            docs,
            docs_source_map,
            outline_mod: _,
            inline_file: _,
            prefix_len: _,
            inline_inner_docs_start: _,
            outline_inner_docs_start: _,
        } = self;
        docs.shrink_to_fit();
        docs_source_map.shrink_to_fit();
    }
}

/// Computes the common leading whitespace before the `*` decoration of a block doc comment,
/// mirroring the block branch of rustdoc's `get_horizontal_trim`. Returns the whitespace prefix
/// (not including the `*`) that every decorated line shares, or `None` when the block has no
/// consistent star column and should be left untouched.
fn block_star_horizontal_trim<'a>(lines: &[&'a str]) -> Option<&'a str> {
    let mut star_col = usize::MAX;
    let mut first = true;

    // Skip the first line (it follows the `/**` prefix) unless it already starts with a star, then
    // ignore leading/trailing blank lines so they don't interfere with detecting the star column.
    let considered = {
        let mut start =
            lines.first().map(|l| if l.trim_start().starts_with('*') { 0 } else { 1 }).unwrap_or(0);
        let mut end = lines.len();
        while start < end && lines[start].trim().is_empty() {
            start += 1;
        }
        while end > start && lines[end - 1].trim().is_empty() {
            end -= 1;
        }
        &lines[start..end]
    };

    for line in considered {
        for (col, ch) in line.chars().enumerate() {
            if col > star_col || !"* \t".contains(ch) {
                return None;
            }
            if ch == '*' {
                if first {
                    star_col = col;
                    first = false;
                } else if star_col != col {
                    return None;
                }
                break;
            }
        }
        if star_col >= line.len() {
            return None;
        }
    }
    // Everything before the star is ASCII whitespace, so `star_col` is a valid byte index.
    let first = *considered.first()?;
    Some(&first[..star_col])
}

/// Strips the leading `*` decoration from a single block-doc line given the shared whitespace
/// `prefix` computed by [`block_star_horizontal_trim`]. Returns the remaining text and the number of
/// bytes removed from the front (all ASCII). Mirrors the per-line stripping in rustdoc's
/// `beautify_doc_string`: after the whitespace prefix, one `*` is removed for the `*`, `* `, and `**`
/// forms, while `*foo` is preserved.
fn strip_block_star_prefix<'a>(line: &'a str, prefix: &str) -> (&'a str, TextSize) {
    let Some(rest) = line.strip_prefix(prefix) else {
        return (line, TextSize::new(0));
    };
    let rest = if rest == "*" || rest.starts_with("* ") || rest.starts_with("**") {
        &rest[1..]
    } else {
        rest
    };
    (rest, TextSize::of(line) - TextSize::of(rest))
}

struct DocMacroExpander<'db> {
    db: &'db dyn SourceDatabase,
    krate: Crate,
    recursion_depth: usize,
    recursion_limit: usize,
    resolver: Resolver<'db>,
    file_id: HirFileId,
    ast_id_map: &'db AstIdMap,
    span_map: SpanMap<'db>,
}

fn expand_doc_expr_via_macro_pipeline<'db>(
    expander: &mut DocMacroExpander<'db>,
    expr: ast::Expr,
) -> Option<String> {
    match expr {
        ast::Expr::ParenExpr(paren_expr) => {
            expand_doc_expr_via_macro_pipeline(expander, paren_expr.expr()?)
        }
        ast::Expr::Literal(literal) => match literal.kind() {
            ast::LiteralKind::String(string) => string.value().ok().map(Into::into),
            _ => None,
        },
        ast::Expr::MacroExpr(macro_expr) => {
            let macro_call = macro_expr.macro_call()?;
            expand_doc_macro_call(expander, macro_call)
        }
        _ => None,
    }
}

fn expand_doc_macro_call<'db>(
    expander: &mut DocMacroExpander<'db>,
    macro_call: ast::MacroCall,
) -> Option<String> {
    if expander.recursion_depth >= expander.recursion_limit {
        return None;
    }

    let path = macro_call.path()?;
    let mod_path = ModPath::from_src(expander.db, path, &mut |range| {
        expander.span_map.span_for_range(range).ctx
    })?;
    let call_site = expander.span_map.span_for_range(macro_call.syntax().text_range());
    let ast_id = AstId::new(expander.file_id, expander.ast_id_map.ast_id(&macro_call));
    let call_id = macro_call_as_call_id(
        expander.db,
        ast_id,
        &mod_path,
        call_site.ctx,
        ExpandTo::Expr,
        expander.krate,
        |path| {
            expander.resolver.resolve_path_as_macro_def(expander.db, path, Some(MacroSubNs::Bang))
        },
        &mut |_, _| (),
    )
    .ok()?
    .value?;

    let (parse, span_map) = &call_id.parse_macro_expansion(expander.db).value;
    let expr = parse.clone().cast::<ast::Expr>().map(|parse| parse.tree())?;

    // Build a new source context for the expansion file so that any further
    // recursive expansion (e.g. a user macro expanding to `concat!(...)`)
    // correctly resolves AstIds and spans in the expansion.
    let expansion_file_id: HirFileId = call_id.into();
    let old_file_id = std::mem::replace(&mut expander.file_id, expansion_file_id);
    let old_span_map =
        std::mem::replace(&mut expander.span_map, SpanMap::ExpansionSpanMap(span_map));
    let old_ast_id_map =
        std::mem::replace(&mut expander.ast_id_map, expansion_file_id.ast_id_map(expander.db));
    expander.recursion_depth += 1;

    let expansion = expand_doc_expr_via_macro_pipeline(expander, expr);

    expander.file_id = old_file_id;
    expander.span_map = old_span_map;
    expander.ast_id_map = old_ast_id_map;
    expander.recursion_depth -= 1;

    expansion
}

fn extend_with_attrs<'a, 'db>(
    result: &mut Docs,
    db: &'db dyn SourceDatabase,
    krate: Crate,
    node: &SyntaxNode,
    file_id: HirFileId,
    expect_inner_attrs: bool,
    indent: &mut usize,
    get_cfg_options: &dyn Fn() -> &'a CfgOptions,
    cfg_options: &mut Option<&'a CfgOptions>,
    make_resolver: &dyn Fn() -> Resolver<'db>,
) {
    // Lazily initialised when we first encounter a `#[doc = macro!()]`.
    let mut expander: Option<DocMacroExpander<'db>> = None;

    expand_cfg_attr_with_doc_comments::<_, Infallible>(
        AttrDocCommentIter::from_syntax_node(node).filter(|attr| match attr {
            Either::Left(attr) => attr.kind().is_inner() == expect_inner_attrs,
            Either::Right(comment) => comment
                .kind()
                .doc
                .is_some_and(|kind| (kind == ast::CommentPlacement::Inner) == expect_inner_attrs),
        }),
        || *cfg_options.get_or_insert_with(get_cfg_options),
        |attr| {
            match attr {
                Either::Right(doc_comment) => result.extend_with_doc_comment(doc_comment, indent),
                Either::Left((attr, _)) => match attr {
                    ast::Meta::KeyValueMeta(attr) if attr.path().is1("doc") => {
                        if let Some(value) = attr.expr() {
                            if let ast::Expr::Literal(value) = &value
                                && let ast::LiteralKind::String(value) = value.kind()
                            {
                                result.extend_with_doc_attr(value, indent);
                            } else {
                                let exp = expander.get_or_insert_with(|| {
                                    let resolver = make_resolver();
                                    let def_map = resolver.top_level_def_map();
                                    let recursion_limit = def_map.recursion_limit() as usize;
                                    DocMacroExpander {
                                        db,
                                        krate,
                                        recursion_depth: 0,
                                        recursion_limit,
                                        resolver,
                                        file_id,
                                        ast_id_map: file_id.ast_id_map(db),
                                        span_map: file_id.span_map(db),
                                    }
                                });
                                if let Some(expanded) =
                                    expand_doc_expr_via_macro_pipeline(exp, value)
                                {
                                    result.extend_with_unmapped_doc_str(&expanded, indent);
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
            ControlFlow::Continue(())
        },
    );
}

pub(crate) fn extract_docs<'a, 'db>(
    db: &'db dyn SourceDatabase,
    krate: Crate,
    resolver: &dyn Fn() -> Resolver<'db>,
    get_cfg_options: &dyn Fn() -> &'a CfgOptions,
    source: InFile<ast::AnyHasAttrs>,
    outer_mod_decl: Option<InFile<ast::Module>>,
    inner_attrs_node: Option<SyntaxNode>,
) -> Option<Box<Docs>> {
    let mut result = Docs {
        docs: String::new(),
        docs_source_map: Vec::new(),
        outline_mod: None,
        inline_file: source.file_id,
        prefix_len: TextSize::new(0),
        inline_inner_docs_start: None,
        outline_inner_docs_start: None,
    };

    let mut cfg_options = None;

    if let Some(outer_mod_decl) = outer_mod_decl {
        let mut indent = usize::MAX;
        // For outer docs (the `mod foo;` declaration), use the module's own resolver.
        extend_with_attrs(
            &mut result,
            db,
            krate,
            outer_mod_decl.value.syntax(),
            outer_mod_decl.file_id,
            false,
            &mut indent,
            get_cfg_options,
            &mut cfg_options,
            resolver,
        );
        result.remove_indent(indent, 0);
        result.outline_mod = Some((outer_mod_decl.file_id, result.docs_source_map.len()));
    }

    let inline_source_map_start = result.docs_source_map.len();
    let mut indent = usize::MAX;
    // For inline docs, use the item's own resolver.
    extend_with_attrs(
        &mut result,
        db,
        krate,
        source.value.syntax(),
        source.file_id,
        false,
        &mut indent,
        get_cfg_options,
        &mut cfg_options,
        resolver,
    );
    if let Some(inner_attrs_node) = &inner_attrs_node {
        result.inline_inner_docs_start = Some(TextSize::of(&result.docs));
        extend_with_attrs(
            &mut result,
            db,
            krate,
            inner_attrs_node,
            source.file_id,
            true,
            &mut indent,
            get_cfg_options,
            &mut cfg_options,
            resolver,
        );
    }
    result.remove_indent(indent, inline_source_map_start);

    result.remove_last_newline();

    result.shrink_to_fit();

    if result.docs.is_empty() { None } else { Some(Box::new(result)) }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hir_expand::InFile;
    use test_fixture::WithFixture;
    use tt::{TextRange, TextSize};

    use crate::test_db::TestDB;

    use super::{Docs, IsInnerDoc};

    #[test]
    fn docs() {
        let (_db, file_id) = TestDB::with_single_file("");
        let mut docs = Docs {
            docs: String::new(),
            docs_source_map: Vec::new(),
            outline_mod: None,
            inline_file: file_id.into(),
            prefix_len: TextSize::new(0),
            inline_inner_docs_start: None,
            outline_inner_docs_start: None,
        };
        let mut indent = usize::MAX;

        let outer = " foo\n\tbar  baz";
        let mut ast_offset = TextSize::new(123);
        for line in outer.split('\n') {
            docs.extend_with_doc_str(line, ast_offset, &mut indent, false);
            ast_offset += TextSize::of(line) + TextSize::of("\n");
        }

        docs.inline_inner_docs_start = Some(TextSize::of(&docs.docs));
        ast_offset += TextSize::new(123);
        let inner = " bar \n baz";
        for line in inner.split('\n') {
            docs.extend_with_doc_str(line, ast_offset, &mut indent, false);
            ast_offset += TextSize::of(line) + TextSize::of("\n");
        }

        assert_eq!(indent, 1);
        expect![[r#"
            [
                DocsSourceMapLine {
                    string_offset: 0,
                    ast_offset: Some(
                        123,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 5,
                    ast_offset: Some(
                        128,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 15,
                    ast_offset: Some(
                        261,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 20,
                    ast_offset: Some(
                        267,
                    ),
                },
            ]
        "#]]
        .assert_debug_eq(&docs.docs_source_map);

        docs.remove_indent(indent, 0);

        assert_eq!(docs.inline_inner_docs_start, Some(TextSize::new(13)));

        assert_eq!(docs.docs, "foo\nbar  baz\nbar\nbaz\n");
        expect![[r#"
            [
                DocsSourceMapLine {
                    string_offset: 0,
                    ast_offset: Some(
                        124,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 4,
                    ast_offset: Some(
                        129,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 13,
                    ast_offset: Some(
                        262,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 17,
                    ast_offset: Some(
                        268,
                    ),
                },
            ]
        "#]]
        .assert_debug_eq(&docs.docs_source_map);

        docs.append(&docs.clone());
        docs.prepend_str("prefix---");
        assert_eq!(docs.docs, "prefix---foo\nbar  baz\nbar\nbaz\nfoo\nbar  baz\nbar\nbaz\n");
        expect![[r#"
            [
                DocsSourceMapLine {
                    string_offset: 0,
                    ast_offset: Some(
                        124,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 4,
                    ast_offset: Some(
                        129,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 13,
                    ast_offset: Some(
                        262,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 17,
                    ast_offset: Some(
                        268,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 21,
                    ast_offset: Some(
                        124,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 25,
                    ast_offset: Some(
                        129,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 34,
                    ast_offset: Some(
                        262,
                    ),
                },
                DocsSourceMapLine {
                    string_offset: 38,
                    ast_offset: Some(
                        268,
                    ),
                },
            ]
        "#]]
        .assert_debug_eq(&docs.docs_source_map);

        let range = |start, end| TextRange::new(TextSize::new(start), TextSize::new(end));
        let in_file = |range| InFile::new(file_id.into(), range);
        assert_eq!(docs.find_ast_range(range(0, 2)), None);
        assert_eq!(docs.find_ast_range(range(8, 10)), None);
        assert_eq!(
            docs.find_ast_range(range(9, 10)),
            Some((in_file(range(124, 125)), IsInnerDoc::No))
        );
        assert_eq!(docs.find_ast_range(range(20, 23)), None);
        assert_eq!(
            docs.find_ast_range(range(23, 25)),
            Some((in_file(range(263, 265)), IsInnerDoc::Yes))
        );
    }

    /// Builds a [`Docs`] from a single block doc comment body (the text between `/**` and `*/`),
    /// running the same normalization as [`super::extract_docs`] for inline docs.
    fn block_doc(doc: &str, ast_offset: u32) -> Docs {
        let (_db, file_id) = TestDB::with_single_file("");
        let mut docs = Docs {
            docs: String::new(),
            docs_source_map: Vec::new(),
            outline_mod: None,
            inline_file: file_id.into(),
            prefix_len: TextSize::new(0),
            inline_inner_docs_start: None,
            outline_inner_docs_start: None,
        };
        let mut indent = usize::MAX;
        docs.extend_with_doc_str(doc, TextSize::new(ast_offset), &mut indent, true);
        docs.remove_indent(indent, 0);
        docs.remove_last_newline();
        docs
    }

    #[test]
    fn block_doc_comment_strips_leading_stars() {
        #[track_caller]
        fn check(doc: &str, expect: expect_test::Expect) {
            expect.assert_eq(&block_doc(doc, 0).docs);
        }

        // Ordinary `* foo` lines have the star and one following space removed.
        check(
            "\n * foo\n * bar\n ",
            expect![[r#"

            foo
            bar
        "#]],
        );
        // A blank `*` line becomes empty.
        check(
            "\n * foo\n *\n * bar\n ",
            expect![[r#"

            foo

            bar
        "#]],
        );
        // Markdown bullets after the decoration star are preserved.
        check(
            "\n * a list:\n *   * one\n *   * two\n ",
            expect![[r#"

            a list:
              * one
              * two
        "#]],
        );
        // `*foo` (no space after the star) is preserved.
        check(
            "\n *foo\n *bar\n ",
            expect![[r#"

            *foo
            *bar
        "#]],
        );
        // `**` has a single star removed.
        check(
            "\n ** foo\n ** bar\n ",
            expect![[r#"

            * foo
            * bar
        "#]],
        );
        // Inconsistent star columns leave the block untouched.
        check(
            "\n * foo\n   * bar\n ",
            expect![[r#"

            * foo
              * bar
        "#]],
        );
        // A block without stars is untouched.
        check(
            "\n foo\n bar\n ",
            expect![[r#"

            foo
            bar
        "#]],
        );
        // Unicode content past the stripped ASCII prefix is preserved.
        check(
            "\n * café ☕\n * über\n ",
            expect![[r#"

            café ☕
            über
        "#]],
        );
        // CRLF line endings are handled like LF.
        check(
            "\r\n * foo\r\n * bar\r\n ",
            expect![[r#"

            foo
            bar
        "#]],
        );
    }

    #[test]
    fn block_doc_comment_source_map_offsets() {
        // `/**` sits at offset 97, so the body (starting with the newline) is at offset 100.
        let docs = block_doc("\n * foo\n * bar\n ", 100);
        assert_eq!(docs.docs, "\nfoo\nbar\n");

        let (_db, file_id) = TestDB::with_single_file("");
        let range = |start, end| TextRange::new(TextSize::new(start), TextSize::new(end));
        let in_file = |range| InFile::new(file_id.into(), range);

        // `foo` in the normalized string maps back to the `foo` in the original source, past the
        // stripped ` * ` decoration.
        assert_eq!(
            docs.find_ast_range(range(1, 4)),
            Some((in_file(range(104, 107)), IsInnerDoc::No))
        );
        // `bar` on the second decorated line maps back correctly as well.
        assert_eq!(
            docs.find_ast_range(range(5, 8)),
            Some((in_file(range(111, 114)), IsInnerDoc::No))
        );
    }
}
