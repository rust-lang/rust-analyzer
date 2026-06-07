//! Defines database & queries for macro expansion.

use base_db::{Crate, SourceDatabase};
use mbe::MatchedArmIndex;
use rustc_hash::FxHashMap;
use span::{AstIdMap, Edition, FIXUP_ERASED_FILE_AST_ID_MARKER, Span, SyntaxContext, TextRange};
use std::borrow::Cow;
use syntax::{AstNode, Parse, SyntaxError, SyntaxNode, SyntaxToken, T, ast};
use syntax_bridge::{DocCommentDesugarMode, syntax_node_to_token_tree};
use triomphe::Arc;

use crate::{
    AstId, BuiltinAttrExpander, BuiltinDeriveExpander, BuiltinFnLikeExpander, EagerCallInfo,
    EagerExpander, EditionedFileId, ExpandError, ExpandResult, ExpandTo, FileRange, HirFileId,
    MacroCallId, MacroCallKind, MacroCallLoc, MacroDefId, MacroDefKind,
    builtin::pseudo_derive_attr_expansion,
    cfg_process::attr_macro_input_to_token_tree,
    declarative::DeclarativeMacroExpander,
    fixup::{self, SyntaxFixupUndoInfo},
    hygiene::{span_with_call_site_ctxt, span_with_def_site_ctxt, span_with_mixed_site_ctxt},
    proc_macro::{CrateProcMacros, CustomProcMacroExpander, ProcMacros},
    span_map::{ExpansionSpanMap, RealSpanMap, SpanMap},
    tt,
};
/// This is just to ensure the types of smart_macro_arg and macro_arg are the same
type MacroArgResult = (tt::TopSubtree, SyntaxFixupUndoInfo, Span);

/// [`MacroArgResult`] with span byte-ranges excluded from equality.
///
/// `macro_arg` changes whenever the source file changes because token spans embed byte offsets
/// that shift even when only whitespace or comments ("trivia") are added. Since trivia is stripped
/// from the token tree before it reaches the proc-macro, the expansion output is structurally
/// identical before and after a trivia-only edit. This type's [`PartialEq`] ignores span ranges so
/// salsa can backdate [`macro_arg_key`] — and therefore [`proc_macro_raw_output`] — on trivia-only
/// edits, skipping the expensive subprocess call.
#[derive(Clone, Debug)]
pub struct MacroArgKey(MacroArgResult);

impl PartialEq for MacroArgKey {
    fn eq(&self, other: &Self) -> bool {
        let (tt1, undo1, span1) = &self.0;
        let (tt2, undo2, span2) = &other.0;
        zero_ranges(tt1) == zero_ranges(tt2)
            && undo1.with_normalized_trees(zero_ranges) == undo2.with_normalized_trees(zero_ranges)
            && span1.anchor == span2.anchor
            && span1.ctx == span2.ctx
    }
}
impl Eq for MacroArgKey {}

/// The raw output of a proc-macro subprocess call together with the input spans used for that run.
///
/// Stored by [`proc_macro_raw_output`] so that [`expand_proc_macro`] can remap stale span
/// byte-ranges in the cached output when [`proc_macro_raw_output`] is backdated on a trivia-only
/// edit. See [`remap_tt_spans`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcMacroRawOutput {
    /// Raw token tree from the subprocess (before [`fixup::reverse_fixups`]).
    pub tt: ExpandResult<tt::TopSubtree>,
    /// Flat DFS-ordered byte-ranges from the macro input fed to this subprocess call.
    /// Each entry is the `range` field of the corresponding span in `macro_arg`, in the same
    /// DFS order as [`collect_ranges`]. Only the range is stored (not the full span) because
    /// anchor and ctx are stable across trivia-only edits and are recovered from the fresh
    /// `macro_arg` at remap time. This is 8 bytes per token vs 20 bytes for a full [`Span`].
    pub input_ranges: Box<[TextRange]>,
}

/// Total limit on the number of tokens produced by any macro invocation.
///
/// If an invocation produces more tokens than this limit, it will not be stored in the database and
/// an error will be emitted.
///
/// Actual max for `analysis-stats .` at some point: 30672.
const TOKEN_LIMIT: usize = 2_097_152;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenExpander<'db> {
    /// Old-style `macro_rules` or the new macros 2.0
    DeclarativeMacro(&'db DeclarativeMacroExpander),
    /// Stuff like `line!` and `file!`.
    BuiltIn(BuiltinFnLikeExpander),
    /// Built-in eagerly expanded fn-like macros (`include!`, `concat!`, etc.)
    BuiltInEager(EagerExpander),
    /// `global_allocator` and such.
    BuiltInAttr(BuiltinAttrExpander),
    /// `derive(Copy)` and such.
    BuiltInDerive(BuiltinDeriveExpander),
    UnimplementedBuiltIn,
    /// The thing we love the most here in rust-analyzer -- procedural macros.
    ProcMacro(CustomProcMacroExpander),
}

#[query_group::query_group]
pub trait ExpandDatabase: SourceDatabase {
    /// The proc macros. Do not use this! Use `proc_macros_for_crate()` instead.
    #[salsa::input]
    fn proc_macros(&self) -> Arc<ProcMacros>;

    /// Incrementality query to prevent queries from directly depending on `ExpandDatabase::proc_macros`.
    #[salsa::invoke(crate::proc_macro::proc_macros_for_crate)]
    fn proc_macros_for_crate(&self, krate: Crate) -> Option<Arc<CrateProcMacros>>;

    #[salsa::invoke(ast_id_map)]
    #[salsa::transparent]
    fn ast_id_map(&self, file_id: HirFileId) -> &AstIdMap;

    #[salsa::transparent]
    fn resolve_span(&self, span: Span) -> FileRange;

    #[salsa::transparent]
    fn parse_or_expand(&self, file_id: HirFileId) -> SyntaxNode;

    /// Implementation for the macro case.
    #[salsa::transparent]
    fn parse_macro_expansion(
        &self,
        macro_file: MacroCallId,
    ) -> &ExpandResult<(Parse<SyntaxNode>, ExpansionSpanMap)>;

    #[salsa::transparent]
    #[salsa::invoke(SpanMap::new)]
    fn span_map(&self, file_id: HirFileId) -> SpanMap<'_>;

    #[salsa::transparent]
    #[salsa::invoke(crate::span_map::expansion_span_map)]
    fn expansion_span_map(&self, file_id: MacroCallId) -> &ExpansionSpanMap;
    #[salsa::invoke(crate::span_map::real_span_map)]
    #[salsa::transparent]
    fn real_span_map(&self, file_id: EditionedFileId) -> &RealSpanMap;

    /// Lowers syntactic macro call to a token tree representation. That's a firewall
    /// query, only typing in the macro call itself changes the returned
    /// subtree.
    #[deprecated = "calling this is incorrect, call `macro_arg_considering_derives` instead"]
    #[salsa::invoke(macro_arg)]
    #[salsa::transparent]
    fn macro_arg(&self, id: MacroCallId) -> &MacroArgResult;

    #[salsa::transparent]
    fn macro_arg_considering_derives<'db>(
        &'db self,
        id: MacroCallId,
        kind: &MacroCallKind,
    ) -> &'db MacroArgResult;

    /// Fetches the expander for this macro.
    #[salsa::transparent]
    #[salsa::invoke(TokenExpander::macro_expander)]
    fn macro_expander(&self, id: MacroDefId) -> TokenExpander<'_>;

    /// Fetches (and compiles) the expander of this decl macro.
    #[salsa::invoke(DeclarativeMacroExpander::expander)]
    #[salsa::transparent]
    fn decl_macro_expander(
        &self,
        def_crate: Crate,
        id: AstId<ast::Macro>,
    ) -> &DeclarativeMacroExpander;

    /// [`macro_arg`] wrapped in [`MacroArgKey`]. Salsa backdates this query on trivia-only edits
    /// because [`MacroArgKey::eq`] ignores span byte-ranges. [`proc_macro_raw_output`] depends on
    /// this query so the subprocess call is also backdated on trivia-only edits.
    #[salsa::invoke(macro_arg_key)]
    #[salsa::transparent]
    fn macro_arg_key(&self, id: MacroCallId) -> &MacroArgKey;

    /// Raw proc-macro subprocess output plus the input byte-ranges used for that run.
    ///
    /// Backdated on trivia-only edits because it depends on [`macro_arg_key`] whose equality
    /// ignores span byte-ranges. On trivia edits [`parse_macro_expansion`] re-runs (because
    /// `macro_arg` changed), calls [`expand_proc_macro`] which reads this backdated query and
    /// remaps its stale spans to current positions — producing a fresh [`ExpansionSpanMap`]
    /// without re-invoking the subprocess.
    #[salsa::invoke(proc_macro_raw_output)]
    #[salsa::transparent]
    fn proc_macro_raw_output(&self, id: MacroCallId) -> &ProcMacroRawOutput;

    /// Retrieves the span to be used for a proc-macro expansions spans.
    /// This is a firewall query as it requires parsing the file, which we don't want proc-macros to
    /// directly depend on as that would cause to frequent invalidations, mainly because of the
    /// parse queries being LRU cached. If they weren't the invalidations would only happen if the
    /// user wrote in the file that defines the proc-macro.
    #[salsa::invoke_interned(proc_macro_span)]
    fn proc_macro_span(&self, fun: AstId<ast::Fn>) -> Span;

    #[salsa::invoke(parse_macro_expansion_error)]
    #[salsa::transparent]
    fn parse_macro_expansion_error(
        &self,
        macro_call: MacroCallId,
    ) -> Option<ExpandResult<Arc<[SyntaxError]>>>;

    #[salsa::transparent]
    fn syntax_context(&self, file: HirFileId, edition: Edition) -> SyntaxContext;
}

#[salsa_macros::interned(no_lifetime, id = span::SyntaxContext, revisions = usize::MAX)]
pub struct SyntaxContextWrapper {
    pub data: SyntaxContext,
}

fn syntax_context(db: &dyn ExpandDatabase, file: HirFileId, edition: Edition) -> SyntaxContext {
    match file {
        HirFileId::FileId(_) => SyntaxContext::root(edition),
        HirFileId::MacroFile(m) => {
            let kind = &m.loc(db).kind;
            db.macro_arg_considering_derives(m, kind).2.ctx
        }
    }
}

fn resolve_span(db: &dyn ExpandDatabase, Span { range, anchor, ctx: _ }: Span) -> FileRange {
    let file_id = EditionedFileId::from_span_file_id(db, anchor.file_id);
    let anchor_offset =
        db.ast_id_map(file_id.into()).get_erased(anchor.ast_id).text_range().start();
    FileRange { file_id, range: range + anchor_offset }
}

/// This expands the given macro call, but with different arguments. This is
/// used for completion, where we want to see what 'would happen' if we insert a
/// token. The `token_to_map` mapped down into the expansion, with the mapped
/// token(s) returned with their priority.
pub fn expand_speculative(
    db: &dyn ExpandDatabase,
    actual_macro_call: MacroCallId,
    speculative_args: &SyntaxNode,
    token_to_map: SyntaxToken,
) -> Option<(SyntaxNode, Vec<(SyntaxToken, u8)>)> {
    let loc = actual_macro_call.loc(db);
    let (_, _, span) = *db.macro_arg_considering_derives(actual_macro_call, &loc.kind);

    let span_map = RealSpanMap::absolute(span.anchor.file_id);
    let span_map = SpanMap::RealSpanMap(&span_map);

    // Build the subtree and token mapping for the speculative args
    let (mut tt, undo_info) = match &loc.kind {
        MacroCallKind::FnLike { .. } => (
            syntax_bridge::syntax_node_to_token_tree(
                speculative_args,
                span_map,
                span,
                if loc.def.is_proc_macro() {
                    DocCommentDesugarMode::ProcMacro
                } else {
                    DocCommentDesugarMode::Mbe
                },
            ),
            SyntaxFixupUndoInfo::NONE,
        ),
        MacroCallKind::Attr { .. } if loc.def.is_attribute_derive() => (
            syntax_bridge::syntax_node_to_token_tree(
                speculative_args,
                span_map,
                span,
                DocCommentDesugarMode::ProcMacro,
            ),
            SyntaxFixupUndoInfo::NONE,
        ),
        MacroCallKind::Derive { derive_macro_id, .. } => {
            let MacroCallKind::Attr { censored_attr_ids: attr_ids, .. } =
                &derive_macro_id.loc(db).kind
            else {
                unreachable!("`derive_macro_id` should be `MacroCallKind::Attr`");
            };
            attr_macro_input_to_token_tree(
                db,
                speculative_args,
                span_map,
                span,
                true,
                attr_ids,
                loc.krate,
            )
        }
        MacroCallKind::Attr { censored_attr_ids: attr_ids, .. } => attr_macro_input_to_token_tree(
            db,
            speculative_args,
            span_map,
            span,
            false,
            attr_ids,
            loc.krate,
        ),
    };

    let attr_arg = match &loc.kind {
        MacroCallKind::Attr { censored_attr_ids: attr_ids, .. } => {
            if loc.def.is_attribute_derive() {
                // for pseudo-derive expansion we actually pass the attribute itself only
                ast::Attr::cast(speculative_args.clone())
                    .and_then(|attr| {
                        if let ast::Meta::TokenTreeMeta(meta) = attr.meta()? {
                            meta.token_tree()
                        } else {
                            None
                        }
                    })
                    .map(|token_tree| {
                        let mut tree = syntax_node_to_token_tree(
                            token_tree.syntax(),
                            span_map,
                            span,
                            DocCommentDesugarMode::ProcMacro,
                        );
                        tree.set_top_subtree_delimiter_kind(tt::DelimiterKind::Invisible);
                        tree.set_top_subtree_delimiter_span(tt::DelimSpan::from_single(span));
                        tree
                    })
            } else {
                // Attributes may have an input token tree, build the subtree and map for this as well
                // then try finding a token id for our token if it is inside this input subtree.
                let item = ast::Item::cast(speculative_args.clone())?;
                let (_, meta) =
                    attr_ids.invoc_attr().find_attr_range_with_source(db, loc.krate, &item);
                if let ast::Meta::TokenTreeMeta(meta) = meta
                    && let Some(tt) = meta.token_tree()
                {
                    let mut attr_arg = syntax_bridge::syntax_node_to_token_tree(
                        tt.syntax(),
                        span_map,
                        span,
                        DocCommentDesugarMode::ProcMacro,
                    );
                    attr_arg.set_top_subtree_delimiter_kind(tt::DelimiterKind::Invisible);
                    Some(attr_arg)
                } else {
                    None
                }
            }
        }
        _ => None,
    };

    // Do the actual expansion, we need to directly expand the proc macro due to the attribute args
    // Otherwise the expand query will fetch the non speculative attribute args and pass those instead.
    let mut speculative_expansion = match loc.def.kind {
        MacroDefKind::ProcMacro(ast, expander, _) => {
            let span = db.proc_macro_span(ast);
            tt.set_top_subtree_delimiter_kind(tt::DelimiterKind::Invisible);
            tt.set_top_subtree_delimiter_span(tt::DelimSpan::from_single(span));
            expander.expand(
                db,
                loc.def.krate,
                loc.krate,
                &tt,
                attr_arg.as_ref(),
                span_with_def_site_ctxt(db, span, actual_macro_call.into(), loc.def.edition),
                span_with_call_site_ctxt(db, span, actual_macro_call.into(), loc.def.edition),
                span_with_mixed_site_ctxt(db, span, actual_macro_call.into(), loc.def.edition),
            )
        }
        MacroDefKind::BuiltInAttr(_, it) if it.is_derive() => {
            pseudo_derive_attr_expansion(&tt, attr_arg.as_ref()?, span)
        }
        MacroDefKind::Declarative(it, _) => db
            .decl_macro_expander(loc.krate, it)
            .expand_unhygienic(db, tt, loc.kind.call_style(), span),
        MacroDefKind::BuiltIn(_, it) => {
            it.expand(db, actual_macro_call, &tt, span).map_err(Into::into)
        }
        MacroDefKind::BuiltInDerive(_, it) => {
            it.expand(db, actual_macro_call, &tt, span).map_err(Into::into)
        }
        MacroDefKind::BuiltInEager(_, it) => {
            it.expand(db, actual_macro_call, &tt, span).map_err(Into::into)
        }
        MacroDefKind::BuiltInAttr(_, it) => it.expand(db, actual_macro_call, &tt, span),
        MacroDefKind::UnimplementedBuiltIn(_) => expand_unimplemented_builtin_macro(span),
    };

    let expand_to = loc.expand_to();

    fixup::reverse_fixups(&mut speculative_expansion.value, &undo_info);
    let (node, rev_tmap) = token_tree_to_syntax_node(db, &speculative_expansion.value, expand_to);

    let syntax_node = node.syntax_node();
    let token = rev_tmap
        .ranges_with_span(span_map.span_for_range(token_to_map.text_range()))
        .filter_map(|(range, ctx)| syntax_node.covering_element(range).into_token().zip(Some(ctx)))
        .map(|(t, ctx)| {
            // prefer tokens of the same kind and text, as well as non opaque marked ones
            // Note the inversion of the score here, as we want to prefer the first token in case
            // of all tokens having the same score
            let ranking = ctx.is_opaque(db) as u8
                + 2 * (t.kind() != token_to_map.kind()) as u8
                + 4 * ((t.text() != token_to_map.text()) as u8);
            (t, ranking)
        })
        .collect();
    Some((node.syntax_node(), token))
}

fn expand_unimplemented_builtin_macro(span: Span) -> ExpandResult<tt::TopSubtree> {
    ExpandResult::new(
        tt::TopSubtree::empty(tt::DelimSpan::from_single(span)),
        ExpandError::other(span, "this built-in macro is not implemented"),
    )
}

#[salsa::tracked(lru = 1024, returns(ref))]
fn ast_id_map(db: &dyn ExpandDatabase, file_id: HirFileId) -> AstIdMap {
    AstIdMap::from_source(&db.parse_or_expand(file_id))
}

/// Main public API -- parses a hir file, not caring whether it's a real
/// file or a macro expansion.
fn parse_or_expand(db: &dyn ExpandDatabase, file_id: HirFileId) -> SyntaxNode {
    match file_id {
        HirFileId::FileId(file_id) => file_id.parse(db).syntax_node(),
        HirFileId::MacroFile(macro_file) => {
            db.parse_macro_expansion(macro_file).value.0.syntax_node()
        }
    }
}

// FIXME: We should verify that the parsed node is one of the many macro node variants we expect
// instead of having it be untyped
#[salsa_macros::tracked(returns(ref), lru = 512)]
fn parse_macro_expansion(
    db: &dyn ExpandDatabase,
    macro_file: MacroCallId,
) -> ExpandResult<(Parse<SyntaxNode>, ExpansionSpanMap)> {
    let _p = tracing::info_span!("parse_macro_expansion").entered();
    let loc = macro_file.loc(db);
    let expand_to = loc.expand_to();
    let mbe::ValueResult { value: (tt, matched_arm), err } = macro_expand(db, macro_file, loc);

    let (parse, mut rev_token_map) = token_tree_to_syntax_node(db, &tt, expand_to);
    rev_token_map.matched_arm = matched_arm;

    ExpandResult { value: (parse, rev_token_map), err }
}

fn parse_macro_expansion_error(
    db: &dyn ExpandDatabase,
    macro_call_id: MacroCallId,
) -> Option<ExpandResult<Arc<[SyntaxError]>>> {
    let e: ExpandResult<Arc<[SyntaxError]>> =
        db.parse_macro_expansion(macro_call_id).as_ref().map(|it| Arc::from(it.0.errors()));
    if e.value.is_empty() && e.err.is_none() { None } else { Some(e) }
}

pub(crate) fn parse_with_map(
    db: &dyn ExpandDatabase,
    file_id: HirFileId,
) -> (Parse<SyntaxNode>, SpanMap<'_>) {
    match file_id {
        HirFileId::FileId(file_id) => {
            (file_id.parse(db).to_syntax(), SpanMap::RealSpanMap(db.real_span_map(file_id)))
        }
        HirFileId::MacroFile(macro_file) => {
            let (parse, map) = &db.parse_macro_expansion(macro_file).value;
            (parse.clone(), SpanMap::ExpansionSpanMap(map))
        }
    }
}

/// This resolves the [MacroCallId] to check if it is a derive macro if so get the [macro_arg] for the derive.
/// Other wise return the [macro_arg] for the macro_call_id.
///
/// This is not connected to the database so it does not cache the result. However, the inner [macro_arg] query is
#[allow(deprecated)] // we are macro_arg_considering_derives
fn macro_arg_considering_derives<'db>(
    db: &'db dyn ExpandDatabase,
    id: MacroCallId,
    kind: &MacroCallKind,
) -> &'db MacroArgResult {
    match kind {
        // Get the macro arg for the derive macro
        MacroCallKind::Derive { derive_macro_id, .. } => db.macro_arg(*derive_macro_id),
        // Normal macro arg
        _ => db.macro_arg(id),
    }
}

#[salsa_macros::tracked(returns(ref))]
fn macro_arg(db: &dyn ExpandDatabase, id: MacroCallId) -> MacroArgResult {
    let loc = id.loc(db);

    if let MacroCallLoc {
        def: MacroDefId { kind: MacroDefKind::BuiltInEager(..), .. },
        kind: MacroCallKind::FnLike { eager: Some(eager), .. },
        ..
    } = &loc
    {
        return (eager.arg.clone(), SyntaxFixupUndoInfo::NONE, eager.span);
    }

    let (parse, map) = parse_with_map(db, loc.kind.file_id());
    let root = parse.syntax_node();

    let (is_derive, censor_item_tree_attr_ids, item_node, span) = match &loc.kind {
        MacroCallKind::FnLike { ast_id, .. } => {
            let node = &ast_id.to_ptr(db).to_node(&root);
            let path_range = node
                .path()
                .map_or_else(|| node.syntax().text_range(), |path| path.syntax().text_range());
            let span = map.span_for_range(path_range);

            let dummy_tt = |kind| {
                (
                    tt::TopSubtree::from_token_trees(
                        tt::Delimiter { open: span, close: span, kind },
                        tt::TokenTreesView::empty(),
                    ),
                    SyntaxFixupUndoInfo::default(),
                    span,
                )
            };

            let Some(tt) = node.token_tree() else {
                return dummy_tt(tt::DelimiterKind::Invisible);
            };
            let first = tt.left_delimiter_token().map(|it| it.kind()).unwrap_or(T!['(']);
            let last = tt.right_delimiter_token().map(|it| it.kind()).unwrap_or(T![.]);

            let mismatched_delimiters = !matches!(
                (first, last),
                (T!['('], T![')']) | (T!['['], T![']']) | (T!['{'], T!['}'])
            );
            if mismatched_delimiters {
                // Don't expand malformed (unbalanced) macro invocations. This is
                // less than ideal, but trying to expand unbalanced  macro calls
                // sometimes produces pathological, deeply nested code which breaks
                // all kinds of things.
                //
                // So instead, we'll return an empty subtree here
                cov_mark::hit!(issue9358_bad_macro_stack_overflow);

                let kind = match first {
                    _ if loc.def.is_proc_macro() => tt::DelimiterKind::Invisible,
                    T!['('] => tt::DelimiterKind::Parenthesis,
                    T!['['] => tt::DelimiterKind::Bracket,
                    T!['{'] => tt::DelimiterKind::Brace,
                    _ => tt::DelimiterKind::Invisible,
                };
                return dummy_tt(kind);
            }

            let mut tt = syntax_bridge::syntax_node_to_token_tree(
                tt.syntax(),
                map,
                span,
                if loc.def.is_proc_macro() {
                    DocCommentDesugarMode::ProcMacro
                } else {
                    DocCommentDesugarMode::Mbe
                },
            );
            if loc.def.is_proc_macro() {
                // proc macros expect their inputs without parentheses, MBEs expect it with them included
                tt.set_top_subtree_delimiter_kind(tt::DelimiterKind::Invisible);
            }
            return (tt, SyntaxFixupUndoInfo::NONE, span);
        }
        // MacroCallKind::Derive should not be here. As we are getting the argument for the derive macro
        MacroCallKind::Derive { .. } => {
            unreachable!("`ExpandDatabase::macro_arg` called with `MacroCallKind::Derive`")
        }
        MacroCallKind::Attr { ast_id, censored_attr_ids: attr_ids, .. } => {
            let node = ast_id.to_ptr(db).to_node(&root);
            let (_, attr) = attr_ids.invoc_attr().find_attr_range_with_source(db, loc.krate, &node);
            let range = attr
                .path()
                .map(|path| path.syntax().text_range())
                .unwrap_or_else(|| attr.syntax().text_range());
            let span = map.span_for_range(range);

            let is_derive = matches!(loc.def.kind, MacroDefKind::BuiltInAttr(_, expander) if expander.is_derive());
            (is_derive, &**attr_ids, node, span)
        }
    };

    let (mut tt, undo_info) = attr_macro_input_to_token_tree(
        db,
        item_node.syntax(),
        map,
        span,
        is_derive,
        censor_item_tree_attr_ids,
        loc.krate,
    );

    if loc.def.is_proc_macro() {
        // proc macros expect their inputs without parentheses, MBEs expect it with them included
        tt.set_top_subtree_delimiter_kind(tt::DelimiterKind::Invisible);
    }

    (tt, undo_info, span)
}

impl<'db> TokenExpander<'db> {
    fn macro_expander(db: &'db dyn ExpandDatabase, id: MacroDefId) -> TokenExpander<'db> {
        match id.kind {
            MacroDefKind::Declarative(ast_id, _) => {
                TokenExpander::DeclarativeMacro(db.decl_macro_expander(id.krate, ast_id))
            }
            MacroDefKind::BuiltIn(_, expander) => TokenExpander::BuiltIn(expander),
            MacroDefKind::BuiltInAttr(_, expander) => TokenExpander::BuiltInAttr(expander),
            MacroDefKind::BuiltInDerive(_, expander) => TokenExpander::BuiltInDerive(expander),
            MacroDefKind::BuiltInEager(_, expander) => TokenExpander::BuiltInEager(expander),
            MacroDefKind::ProcMacro(_, expander, _) => TokenExpander::ProcMacro(expander),
            MacroDefKind::UnimplementedBuiltIn(_) => TokenExpander::UnimplementedBuiltIn,
        }
    }
}

fn macro_expand<'db>(
    db: &'db dyn ExpandDatabase,
    macro_call_id: MacroCallId,
    loc: &MacroCallLoc,
) -> ExpandResult<(Cow<'db, tt::TopSubtree>, MatchedArmIndex)> {
    let _p = tracing::info_span!("macro_expand").entered();

    let (ExpandResult { value: (tt, matched_arm), err }, span) = match loc.def.kind {
        MacroDefKind::ProcMacro(..) => {
            // expand_proc_macro is not a salsa query — called directly to avoid
            // caching a second copy of the TT (proc_macro_raw_output already caches it).
            return expand_proc_macro(db, macro_call_id).map(|it| (Cow::Owned(it), None));
        }
        _ => {
            let (macro_arg, undo_info, span) =
                db.macro_arg_considering_derives(macro_call_id, &loc.kind);
            let span = *span;

            let arg = macro_arg;
            let res = match loc.def.kind {
                MacroDefKind::Declarative(id, _) => db
                    .decl_macro_expander(loc.def.krate, id)
                    .expand(db, arg.clone(), macro_call_id, span),
                MacroDefKind::BuiltIn(_, it) => {
                    it.expand(db, macro_call_id, arg, span).map_err(Into::into).zip_val(None)
                }
                MacroDefKind::BuiltInDerive(_, it) => {
                    it.expand(db, macro_call_id, arg, span).map_err(Into::into).zip_val(None)
                }
                MacroDefKind::UnimplementedBuiltIn(_) => {
                    expand_unimplemented_builtin_macro(span).zip_val(None)
                }
                MacroDefKind::BuiltInEager(_, it) => {
                    // This might look a bit odd, but we do not expand the inputs to eager macros here.
                    // Eager macros inputs are expanded, well, eagerly when we collect the macro calls.
                    // That kind of expansion uses the ast id map of an eager macros input though which goes through
                    // the HirFileId machinery. As eager macro inputs are assigned a macro file id that query
                    // will end up going through here again, whereas we want to just want to inspect the raw input.
                    // As such we just return the input subtree here.
                    let eager = match &loc.kind {
                        MacroCallKind::FnLike { eager: None, .. } => {
                            return ExpandResult::ok(Cow::Borrowed(macro_arg)).zip_val(None);
                        }
                        MacroCallKind::FnLike { eager: Some(eager), .. } => Some(&**eager),
                        _ => None,
                    };

                    let mut res = it.expand(db, macro_call_id, arg, span).map_err(Into::into);

                    if let Some(EagerCallInfo { error, .. }) = eager {
                        // FIXME: We should report both errors!
                        res.err = error.clone().or(res.err);
                    }
                    res.zip_val(None)
                }
                MacroDefKind::BuiltInAttr(_, it) => {
                    let mut res = it.expand(db, macro_call_id, arg, span);
                    fixup::reverse_fixups(&mut res.value, undo_info);
                    res.zip_val(None)
                }
                MacroDefKind::ProcMacro(_, _, _) => unreachable!(),
            };
            (ExpandResult { value: res.value, err: res.err }, span)
        }
    };

    // Skip checking token tree limit for include! macro call
    if !loc.def.is_include() {
        // Set a hard limit for the expanded tt
        if let Err(value) = check_tt_count(&tt) {
            return value
                .map(|()| Cow::Owned(tt::TopSubtree::empty(tt::DelimSpan::from_single(span))))
                .zip_val(matched_arm);
        }
    }

    ExpandResult { value: (Cow::Owned(tt), matched_arm), err }
}

fn proc_macro_span(db: &dyn ExpandDatabase, ast: AstId<ast::Fn>) -> Span {
    let root = db.parse_or_expand(ast.file_id);
    let ast_id_map = &db.ast_id_map(ast.file_id);
    let span_map = &db.span_map(ast.file_id);

    let node = ast_id_map.get(ast.value).to_node(&root);
    let range = ast::HasName::name(&node)
        .map_or_else(|| node.syntax().text_range(), |name| name.syntax().text_range());
    span_map.span_for_range(range)
}

/// Implements [`ExpandDatabase::macro_arg_key`].
#[salsa_macros::tracked(returns(ref))]
fn macro_arg_key(db: &dyn ExpandDatabase, id: MacroCallId) -> MacroArgKey {
    MacroArgKey(db.macro_arg(id).clone())
}

#[salsa_macros::tracked(returns(ref))]
fn proc_macro_raw_output(db: &dyn ExpandDatabase, id: MacroCallId) -> ProcMacroRawOutput {
    let loc = id.loc(db);

    // Calling macro_arg_key here creates a salsa dependency that uses MacroArgKey's
    // position-ignoring PartialEq, so this query is backdated on trivia-only edits.
    let MacroArgKey((macro_arg, _undo_info, _call_span)) = match &loc.kind {
        MacroCallKind::Derive { derive_macro_id, .. } => db.macro_arg_key(*derive_macro_id),
        _ => db.macro_arg_key(id),
    };

    let input_ranges = collect_ranges(macro_arg);

    let (ast, expander) = match loc.def.kind {
        MacroDefKind::ProcMacro(ast, expander, _) => (ast, expander),
        _ => unreachable!(),
    };

    let attr_arg = match &loc.kind {
        MacroCallKind::Attr { attr_args: Some(attr_args), .. } => Some(&**attr_args),
        _ => None,
    };

    let tt = {
        let span = db.proc_macro_span(ast);
        expander.expand(
            db,
            loc.def.krate,
            loc.krate,
            macro_arg,
            attr_arg,
            span_with_def_site_ctxt(db, span, id.into(), loc.def.edition),
            span_with_call_site_ctxt(db, span, id.into(), loc.def.edition),
            span_with_mixed_site_ctxt(db, span, id.into(), loc.def.edition),
        )
    };

    ProcMacroRawOutput { tt, input_ranges }
}

fn expand_proc_macro(db: &dyn ExpandDatabase, id: MacroCallId) -> ExpandResult<tt::TopSubtree> {
    let loc = id.loc(db);

    // Fresh macro_arg: this direct dependency on macro_arg_considering_derives (which is NOT
    // backdated on trivia edits) ensures that expand_proc_macro re-runs on trivia edits even
    // when proc_macro_raw_output is backdated. The re-run remaps the stale output spans to fresh
    // positions, keeping ExpansionSpanMap current without re-invoking the subprocess.
    let (fresh_macro_arg, fresh_undo_info, span) = db.macro_arg_considering_derives(id, &loc.kind);

    // Raw output from the subprocess — may be a backdated cached result on trivia-only edits.
    let raw = db.proc_macro_raw_output(id);

    // Remap stale span byte-ranges in the cached TT to the current source positions.
    // On structural edits proc_macro_raw_output re-ran with fresh input, so raw.input_ranges
    // and the fresh macro_arg ranges are identical and remap_tt_spans is a structural no-op.
    //
    // We reconstruct each old span as `Span { range: old_range, anchor: fresh.anchor, ctx:
    // fresh.ctx }` because anchor and ctx are stable across trivia-only edits (only byte-range
    // positions shift), and on structural edits the raw output was recomputed so old == fresh.
    let fresh_input_spans = collect_spans(fresh_macro_arg);
    // Build a map from old span → new span for every input token whose byte-range shifted.
    // Entries where old_range == fresh.range are skipped (no-ops): tokens before the trivia
    // insertion point and all tokens on structural edits (where input_ranges == fresh ranges).
    // This keeps the map small and lets remap_tt_spans early-exit via `map.is_empty()` on
    // structural edits, avoiding a pointless TT clone.
    let span_remap: FxHashMap<Span, Span> = raw
        .input_ranges
        .iter()
        .zip(fresh_input_spans.iter())
        .filter(|(_, fresh)| fresh.anchor.ast_id != FIXUP_ERASED_FILE_AST_ID_MARKER)
        .filter(|(old_range, fresh)| **old_range != fresh.range)
        .map(|(&old_range, &fresh)| {
            // Reconstruct the old span using old range + fresh anchor/ctx.
            // anchor and ctx are stable across trivia-only edits; on structural edits
            // old_range == fresh.range so this branch is never reached.
            let old = Span { range: old_range, anchor: fresh.anchor, ctx: fresh.ctx };
            (old, fresh)
        })
        .collect();

    // Set a hard limit for the expanded tt
    if let Err(err_val) = check_tt_count(&raw.tt.value) {
        return err_val.map(|()| tt::TopSubtree::empty(tt::DelimSpan::from_single(*span)));
    }

    let err = raw.tt.err.clone();
    let mut tt = remap_tt_spans(raw.tt.value.clone(), &span_remap);

    fixup::reverse_fixups(&mut tt, fresh_undo_info);

    ExpandResult { value: tt, err }
}

pub(crate) fn token_tree_to_syntax_node(
    db: &dyn ExpandDatabase,
    tt: &tt::TopSubtree,
    expand_to: ExpandTo,
) -> (Parse<SyntaxNode>, ExpansionSpanMap) {
    let entry_point = match expand_to {
        ExpandTo::Statements => syntax_bridge::TopEntryPoint::MacroStmts,
        ExpandTo::Items => syntax_bridge::TopEntryPoint::MacroItems,
        ExpandTo::Pattern => syntax_bridge::TopEntryPoint::Pattern,
        ExpandTo::Type => syntax_bridge::TopEntryPoint::Type,
        ExpandTo::Expr => syntax_bridge::TopEntryPoint::Expr,
    };
    syntax_bridge::token_tree_to_syntax_node(tt, entry_point, &mut |ctx| ctx.edition(db))
}

/// Collects the byte-range from every span in a token tree, in depth-first order
/// (open delimiter, children, close delimiter). Stores only the [`TextRange`] field of each
/// [`Span`] — anchor and ctx are stable across trivia-only edits and are recovered from a
/// same-structure fresh token tree at remap time. 8 bytes per token instead of 20.
fn collect_ranges(tt: &tt::TopSubtree) -> Box<[TextRange]> {
    fn recurse(iter: tt::TtIter<'_>, out: &mut Vec<TextRange>) {
        for el in iter {
            match el {
                tt::TtElement::Leaf(l) => out.push(l.span().range),
                tt::TtElement::Subtree(sub, children) => {
                    out.push(sub.delimiter.open.range);
                    recurse(children, out);
                    out.push(sub.delimiter.close.range);
                }
            }
        }
    }
    let top = tt.top_subtree();
    let mut out = Vec::new();
    out.push(top.delimiter.open.range);
    recurse(tt.iter(), &mut out);
    out.push(top.delimiter.close.range);
    out.into_boxed_slice()
}

/// Collects all spans in a token tree in depth-first order (open delimiter, children, close
/// delimiter). The resulting slice is aligned index-for-index with the spans of a structurally
/// identical token tree, enabling O(n) span remapping without re-running the macro.
fn collect_spans(tt: &tt::TopSubtree) -> Box<[Span]> {
    fn recurse(iter: tt::TtIter<'_>, out: &mut Vec<Span>) {
        for el in iter {
            match el {
                tt::TtElement::Leaf(l) => out.push(*l.span()),
                tt::TtElement::Subtree(sub, children) => {
                    out.push(sub.delimiter.open);
                    recurse(children, out);
                    out.push(sub.delimiter.close);
                }
            }
        }
    }
    let top = tt.top_subtree();
    let mut out = Vec::new();
    out.push(top.delimiter.open);
    recurse(tt.iter(), &mut out);
    out.push(top.delimiter.close);
    out.into_boxed_slice()
}

/// Rebuilds `tree` replacing each span that appears in `map` with its mapped value.
/// Spans not present in the map (def-site, call-site, or macro-generated spans that don't
/// originate from the call-site input) are left unchanged.
fn remap_tt_spans(tree: tt::TopSubtree, map: &FxHashMap<Span, Span>) -> tt::TopSubtree {
    fn remap(span: Span, map: &FxHashMap<Span, Span>) -> Span {
        // Fixup spans encode an index in range.start(); never remap them.
        if span.anchor.ast_id == FIXUP_ERASED_FILE_AST_ID_MARKER {
            return span;
        }
        map.get(&span).copied().unwrap_or(span)
    }

    fn leaf(mut l: tt::Leaf, map: &FxHashMap<Span, Span>) -> tt::Leaf {
        match &mut l {
            tt::Leaf::Literal(it) => it.span = remap(it.span, map),
            tt::Leaf::Punct(it) => it.span = remap(it.span, map),
            tt::Leaf::Ident(it) => it.span = remap(it.span, map),
        }
        l
    }

    fn build(
        builder: &mut tt::TopSubtreeBuilder,
        iter: tt::TtIter<'_>,
        map: &FxHashMap<Span, Span>,
    ) {
        for el in iter {
            match el {
                tt::TtElement::Leaf(l) => builder.push(leaf(l, map)),
                tt::TtElement::Subtree(sub, children) => {
                    builder.open(sub.delimiter.kind, remap(sub.delimiter.open, map));
                    build(builder, children, map);
                    builder.close(remap(sub.delimiter.close, map));
                }
            }
        }
    }

    if map.is_empty() {
        return tree;
    }

    let top = tree.top_subtree();
    let mut b = tt::TopSubtreeBuilder::new(tt::Delimiter {
        open: remap(top.delimiter.open, map),
        close: remap(top.delimiter.close, map),
        kind: top.delimiter.kind,
    });
    build(&mut b, tree.iter(), map);
    b.build()
}

/// Rebuilds `tree` with all span byte-ranges set to zero, preserving fixup spans whose
/// `range.start()` encodes an index into [`SyntaxFixupUndoInfo::original`].
/// Used by [`MacroArgKey::eq`] to compare token trees ignoring position information.
fn zero_ranges(tree: &tt::TopSubtree) -> tt::TopSubtree {
    fn zero(span: Span) -> Span {
        if span.anchor.ast_id == FIXUP_ERASED_FILE_AST_ID_MARKER {
            return span;
        }
        use syntax::TextSize;
        Span { range: TextRange::empty(TextSize::new(0)), ..span }
    }
    fn leaf(mut l: tt::Leaf) -> tt::Leaf {
        match &mut l {
            tt::Leaf::Literal(it) => it.span = zero(it.span),
            tt::Leaf::Punct(it) => it.span = zero(it.span),
            tt::Leaf::Ident(it) => it.span = zero(it.span),
        }
        l
    }
    fn build(builder: &mut tt::TopSubtreeBuilder, iter: tt::TtIter<'_>) {
        for el in iter {
            match el {
                tt::TtElement::Leaf(l) => builder.push(leaf(l)),
                tt::TtElement::Subtree(sub, children) => {
                    builder.open(sub.delimiter.kind, zero(sub.delimiter.open));
                    build(builder, children);
                    builder.close(zero(sub.delimiter.close));
                }
            }
        }
    }
    let top = tree.top_subtree();
    let mut b = tt::TopSubtreeBuilder::new(tt::Delimiter {
        open: zero(top.delimiter.open),
        close: zero(top.delimiter.close),
        kind: top.delimiter.kind,
    });
    build(&mut b, tree.iter());
    b.build()
}

fn check_tt_count(tt: &tt::TopSubtree) -> Result<(), ExpandResult<()>> {
    let tt = tt.top_subtree();
    let count = tt.count();
    if count <= TOKEN_LIMIT {
        Ok(())
    } else {
        Err(ExpandResult {
            value: (),
            err: Some(ExpandError::other(
                tt.delimiter.open,
                format!(
                    "macro invocation exceeds token limit: produced {count} tokens, limit is {TOKEN_LIMIT}",
                ),
            )),
        })
    }
}
