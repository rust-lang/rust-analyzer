//! "Recursive" Syntax highlighting for code in doctests and fixtures.

use std::mem;

use either::Either;
use hir::{EditionedFileId, HirFileId, InFile, Semantics, sym};
use ide_db::{
    SymbolKind, active_parameter::ActiveParameter, base_db::salsa, defs::Definition,
    documentation::docs_with_rangemap, rust_doc::is_rust_fence,
};
use syntax::{
    AstToken, NodeOrToken, SyntaxNode, TextRange, TextSize,
    ast::{self, AstNode, IsString, QuoteOffsets},
};
use test_utils::MiniCore;

use crate::{
    Analysis, HlMod, HlRange, HlTag, RootDatabase,
    doc_links::{doc_attributes, extract_definitions_from_docs, resolve_doc_path_for_def},
    syntax_highlighting::{HighlightConfig, highlights::Highlights, injector::Injector},
};

pub(super) fn ra_fixture(
    hl: &mut Highlights,
    sema: &Semantics<'_, RootDatabase>,
    config: &HighlightConfig<'_>,
    literal: &ast::String,
    expanded: &ast::String,
) -> Option<()> {
    let active_parameter =
        salsa::attach(sema.db, || ActiveParameter::at_token(sema, expanded.syntax().clone()))?;
    let has_rust_fixture_attr = active_parameter.attrs().is_some_and(|attrs| {
        attrs.filter_map(|attr| attr.as_simple_path()).any(|path| {
            path.segments()
                .zip(["rust_analyzer", "rust_fixture"])
                .all(|(seg, name)| seg.name_ref().map_or(false, |nr| nr.text() == name))
        })
    });
    if !has_rust_fixture_attr {
        return None;
    }
    let value = literal.value().ok()?;

    if let Some(range) = literal.open_quote_text_range() {
        hl.add(HlRange { range, highlight: HlTag::StringLiteral.into(), binding_hash: None })
    }

    let minicore = config.minicore.unwrap_or(MiniCore::RAW_SOURCE);

    let mut inj = Injector::default();

    // This is used for the `Injector`, to resolve precise location in the string literal,
    // which will then be used to resolve precise location in the enclosing file.
    let mut offset_with_indent = TextSize::new(0);
    // This is used to resolve the location relative to the virtual file into a location
    // relative to the indentation-trimmed file which will then (by the `Injector`) used
    // to resolve to a location in the actual file.
    // Besides indentation, we also skip `$0` cursors for this, since they are not included
    // in the virtual files.
    let mut offset_without_indent = TextSize::new(0);

    let mut text = &*value;
    if let Some(t) = text.strip_prefix('\n') {
        offset_with_indent += TextSize::of("\n");
        text = t;
    }
    // This stores the offsets of each line, **after we remove indentation**.
    let mut line_offsets = Vec::new();
    for mut line in text.split_inclusive('\n') {
        line_offsets.push(offset_without_indent);

        if line.starts_with("@@") {
            // Introducing `//` into a fixture inside fixture causes all sorts of problems,
            // so for testing purposes we escape it as `@@` and replace it here.
            inj.add("//", TextRange::at(offset_with_indent, TextSize::of("@@")));
            line = &line["@@".len()..];
            offset_with_indent += TextSize::of("@@");
            offset_without_indent += TextSize::of("@@");
        }

        // Remove indentation to simplify the mapping with fixture (which de-indents).
        // Removing indentation shouldn't affect highlighting.
        let mut unindented_line = line.trim_start();
        if unindented_line.is_empty() {
            // The whole line was whitespaces, but we need the newline.
            unindented_line = "\n";
        }
        offset_with_indent += TextSize::of(line) - TextSize::of(unindented_line);

        let marker = "$0";
        match unindented_line.find(marker) {
            Some(marker_pos) => {
                let (before_marker, after_marker) = unindented_line.split_at(marker_pos);
                let after_marker = &after_marker[marker.len()..];

                inj.add(
                    before_marker,
                    TextRange::at(offset_with_indent, TextSize::of(before_marker)),
                );
                offset_with_indent += TextSize::of(before_marker);
                offset_without_indent += TextSize::of(before_marker);

                if let Some(marker_range) =
                    literal.map_range_up(TextRange::at(offset_with_indent, TextSize::of(marker)))
                {
                    hl.add(HlRange {
                        range: marker_range,
                        highlight: HlTag::Keyword | HlMod::Injected,
                        binding_hash: None,
                    });
                }
                offset_with_indent += TextSize::of(marker);

                inj.add(
                    after_marker,
                    TextRange::at(offset_with_indent, TextSize::of(after_marker)),
                );
                offset_with_indent += TextSize::of(after_marker);
                offset_without_indent += TextSize::of(after_marker);
            }
            None => {
                inj.add(
                    unindented_line,
                    TextRange::at(offset_with_indent, TextSize::of(unindented_line)),
                );
                offset_with_indent += TextSize::of(unindented_line);
                offset_without_indent += TextSize::of(unindented_line);
            }
        }
    }

    let (analysis, tmp_file_ids) = Analysis::from_ra_fixture(&inj.take_text(), minicore);

    for (tmp_file_id, tmp_file_line) in tmp_file_ids {
        // This could be `None` if the file is empty.
        let Some(&tmp_file_offset) = line_offsets.get(tmp_file_line) else {
            continue;
        };
        for mut hl_range in analysis
            .highlight(
                HighlightConfig {
                    syntactic_name_ref_highlighting: false,
                    punctuation: true,
                    operator: true,
                    strings: true,
                    specialize_punctuation: config.specialize_punctuation,
                    specialize_operator: config.operator,
                    inject_doc_comment: config.inject_doc_comment,
                    macro_bang: config.macro_bang,
                    // What if there is a fixture inside a fixture? It's fixtures all the way down.
                    // (In fact, we have a fixture inside a fixture in our test suite!)
                    minicore: config.minicore,
                },
                tmp_file_id,
            )
            .unwrap()
        {
            // Resolve the offset relative to the virtual file to an offset relative to the combined indentation-trimmed file
            let range = hl_range.range + tmp_file_offset;
            // Then resolve that to an offset relative to the real file.
            for range in inj.map_range_up(range) {
                // And finally resolve the offset relative to the literal to relative to the file.
                if let Some(range) = literal.map_range_up(range) {
                    hl_range.range = range;
                    hl_range.highlight |= HlMod::Injected;
                    hl.add(hl_range);
                }
            }
        }
    }

    if let Some(range) = literal.close_quote_text_range() {
        hl.add(HlRange { range, highlight: HlTag::StringLiteral.into(), binding_hash: None })
    }

    Some(())
}

const RUSTDOC_FENCE_LENGTH: usize = 3;
const RUSTDOC_FENCES: [&str; 2] = ["```", "~~~"];

/// Injection of syntax highlighting of doctests and intra doc links.
pub(super) fn doc_comment(
    hl: &mut Highlights,
    sema: &Semantics<'_, RootDatabase>,
    config: &HighlightConfig<'_>,
    src_file_id: EditionedFileId,
    node: &SyntaxNode,
) {
    let (attributes, def) = match doc_attributes(sema, node) {
        Some(it) => it,
        None => return,
    };
    let src_file_id: HirFileId = src_file_id.into();

    // Extract intra-doc links and emit highlights for them.
    if let Some((docs, doc_mapping)) = docs_with_rangemap(sema.db, &attributes) {
        salsa::attach(sema.db, || {
            extract_definitions_from_docs(&docs)
                .into_iter()
                .filter_map(|(range, link, ns)| {
                    doc_mapping
                        .map(range)
                        .filter(|(mapping, _)| mapping.file_id == src_file_id)
                        .and_then(|(InFile { value: mapped_range, .. }, attr_id)| {
                            Some(mapped_range).zip(resolve_doc_path_for_def(
                                sema.db,
                                def,
                                &link,
                                ns,
                                attr_id.is_inner_attr(),
                            ))
                        })
                })
                .for_each(|(range, def)| {
                    hl.add(HlRange {
                        range,
                        highlight: module_def_to_hl_tag(def)
                            | HlMod::Documentation
                            | HlMod::Injected
                            | HlMod::IntraDocLink,
                        binding_hash: None,
                    })
                })
        });
    }

    // Extract doc-test sources from the docs and calculate highlighting for them.

    let mut inj = Injector::default();
    inj.add_unmapped("fn doctest() {\n");

    let attrs_source_map = attributes.source_map(sema.db);

    let mut is_codeblock = false;
    let mut is_doctest = false;

    let mut new_comments = Vec::new();
    let mut string;

    for attr in attributes.by_key(sym::doc).attrs() {
        let InFile { file_id, value: src } = attrs_source_map.source_of(attr);
        if file_id != src_file_id {
            continue;
        }
        let (line, range) = match &src {
            Either::Left(it) => {
                string = match find_doc_string_in_attr(attr, it) {
                    Some(it) => it,
                    None => continue,
                };
                let text = string.text();
                let text_range = string.syntax().text_range();
                match string.quote_offsets() {
                    Some(QuoteOffsets { contents, .. }) => {
                        (&text[contents - text_range.start()], contents)
                    }
                    None => (text, text_range),
                }
            }
            Either::Right(comment) => {
                let value = comment.prefix().len();
                let range = comment.syntax().text_range();
                (
                    &comment.text()[value..],
                    TextRange::new(range.start() + TextSize::try_from(value).unwrap(), range.end()),
                )
            }
        };

        let mut range_start = range.start();
        for line in line.split('\n') {
            let line_len = TextSize::from(line.len() as u32);
            let prev_range_start = {
                let next_range_start = range_start + line_len + TextSize::from(1);
                mem::replace(&mut range_start, next_range_start)
            };
            let mut pos = TextSize::from(0);

            match RUSTDOC_FENCES.into_iter().find_map(|fence| line.find(fence)) {
                Some(idx) => {
                    is_codeblock = !is_codeblock;
                    // Check whether code is rust by inspecting fence guards
                    let guards = &line[idx + RUSTDOC_FENCE_LENGTH..];
                    let is_rust = is_rust_fence(guards);
                    is_doctest = is_codeblock && is_rust;
                    continue;
                }
                None if !is_doctest => continue,
                None => (),
            }

            // whitespace after comment is ignored
            if let Some(ws) = line[pos.into()..].chars().next().filter(|c| c.is_whitespace()) {
                pos += TextSize::of(ws);
            }
            // lines marked with `#` should be ignored in output, we skip the `#` char
            if line[pos.into()..].starts_with('#') {
                pos += TextSize::of('#');
            }

            new_comments.push(TextRange::at(prev_range_start, pos));
            inj.add(&line[pos.into()..], TextRange::new(pos, line_len) + prev_range_start);
            inj.add_unmapped("\n");
        }
    }

    if new_comments.is_empty() {
        return; // no need to run an analysis on an empty file
    }

    inj.add_unmapped("\n}");

    let (analysis, tmp_file_id) = Analysis::from_single_file(inj.take_text());

    if let Ok(ranges) = analysis.with_db(|db| {
        super::highlight(
            db,
            &HighlightConfig {
                syntactic_name_ref_highlighting: true,
                punctuation: true,
                operator: true,
                strings: true,
                specialize_punctuation: config.specialize_punctuation,
                specialize_operator: config.operator,
                inject_doc_comment: config.inject_doc_comment,
                macro_bang: config.macro_bang,
                minicore: config.minicore,
            },
            tmp_file_id,
            None,
        )
    }) {
        for HlRange { range, highlight, binding_hash } in ranges {
            for range in inj.map_range_up(range) {
                hl.add(HlRange { range, highlight: highlight | HlMod::Injected, binding_hash });
            }
        }
    }

    for range in new_comments {
        hl.add(HlRange {
            range,
            highlight: HlTag::Comment | HlMod::Documentation,
            binding_hash: None,
        });
    }
}

fn find_doc_string_in_attr(attr: &hir::Attr, it: &ast::Attr) -> Option<ast::String> {
    match it.expr() {
        // #[doc = lit]
        Some(ast::Expr::Literal(lit)) => match lit.kind() {
            ast::LiteralKind::String(it) => Some(it),
            _ => None,
        },
        // #[cfg_attr(..., doc = "", ...)]
        None => {
            // We gotta hunt the string token manually here
            let text = attr.string_value()?.as_str();
            // FIXME: We just pick the first string literal that has the same text as the doc attribute
            // This means technically we might highlight the wrong one
            it.syntax()
                .descendants_with_tokens()
                .filter_map(NodeOrToken::into_token)
                .filter_map(ast::String::cast)
                .find(|string| string.text().get(1..string.text().len() - 1) == Some(text))
        }
        _ => None,
    }
}

fn module_def_to_hl_tag(def: Definition) -> HlTag {
    let symbol = match def {
        Definition::Module(_) | Definition::Crate(_) | Definition::ExternCrateDecl(_) => {
            SymbolKind::Module
        }
        Definition::Function(_) => SymbolKind::Function,
        Definition::Adt(hir::Adt::Struct(_)) => SymbolKind::Struct,
        Definition::Adt(hir::Adt::Enum(_)) => SymbolKind::Enum,
        Definition::Adt(hir::Adt::Union(_)) => SymbolKind::Union,
        Definition::Variant(_) => SymbolKind::Variant,
        Definition::Const(_) => SymbolKind::Const,
        Definition::Static(_) => SymbolKind::Static,
        Definition::Trait(_) => SymbolKind::Trait,
        Definition::TypeAlias(_) => SymbolKind::TypeAlias,
        Definition::BuiltinLifetime(_) => SymbolKind::LifetimeParam,
        Definition::BuiltinType(_) => return HlTag::BuiltinType,
        Definition::Macro(_) => SymbolKind::Macro,
        Definition::Field(_) | Definition::TupleField(_) => SymbolKind::Field,
        Definition::SelfType(_) => SymbolKind::Impl,
        Definition::Local(_) => SymbolKind::Local,
        Definition::GenericParam(gp) => match gp {
            hir::GenericParam::TypeParam(_) => SymbolKind::TypeParam,
            hir::GenericParam::ConstParam(_) => SymbolKind::ConstParam,
            hir::GenericParam::LifetimeParam(_) => SymbolKind::LifetimeParam,
        },
        Definition::Label(_) => SymbolKind::Label,
        Definition::BuiltinAttr(_) => SymbolKind::BuiltinAttr,
        Definition::ToolModule(_) => SymbolKind::ToolModule,
        Definition::DeriveHelper(_) => SymbolKind::DeriveHelper,
        Definition::InlineAsmRegOrRegClass(_) => SymbolKind::InlineAsmRegOrRegClass,
        Definition::InlineAsmOperand(_) => SymbolKind::Local,
    };
    HlTag::Symbol(symbol)
}
