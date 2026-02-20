//! See [`complete_fn_param`].

use std::fmt::Write;

use hir::HirDisplay;
use ide_db::{FxHashMap, FxHashSet, active_parameter::callable_for_token};
use itertools::Either;
use syntax::{
    AstNode, Direction, SmolStr, SyntaxKind, TextRange, TextSize, ToSmolStr, algo,
    ast::{self, HasModuleItem},
    format_smolstr, match_ast,
};

use crate::{
    CompletionContext, CompletionItem, CompletionItemKind, Completions,
    context::{ParamContext, ParamKind, PatternContext},
};

// FIXME: Make this a submodule of [`pattern`]
/// Complete repeated parameters, both name and type. For example, if all
/// functions in a file have a `spam: &mut Spam` parameter, a completion with
/// `spam: &mut Spam` insert text/label will be suggested.
///
/// Also complete parameters for closure or local functions from the surrounding defined locals.
pub(crate) fn complete_fn_param(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
    pattern_ctx: &PatternContext,
) -> Option<()> {
    let (ParamContext { param_list, kind, param, .. }, impl_or_trait) = match pattern_ctx {
        PatternContext { param_ctx: Some(kind), impl_or_trait, .. } => (kind, impl_or_trait),
        _ => return None,
    };

    let comma_wrapper = comma_wrapper(ctx);
    let mut add_new_item_to_acc = |label: &str| {
        let mk_item = |label: &str, range: TextRange| {
            CompletionItem::new(CompletionItemKind::Binding, range, label, ctx.edition)
        };
        let item = match &comma_wrapper {
            Some((fmt, range)) => mk_item(&fmt(label), *range),
            None => mk_item(label, ctx.source_range()),
        };
        // Completion lookup is omitted intentionally here.
        // See the full discussion: https://github.com/rust-lang/rust-analyzer/issues/12073
        item.add_to(acc, ctx.db)
    };

    match kind {
        ParamKind::Function(function) => {
            fill_fn_params(ctx, function, param_list, param, impl_or_trait, add_new_item_to_acc);
        }
        ParamKind::Closure(closure) => {
            if is_simple_param(param) {
                let stmt_list = closure.syntax().ancestors().find_map(ast::StmtList::cast)?;
                params_from_stmt_list_scope(ctx, stmt_list, |name, ty| {
                    add_new_item_to_acc(&format_smolstr!(
                        "{}: {ty}",
                        name.display(ctx.db, ctx.edition)
                    ));
                });
            }
        }
    }

    Some(())
}

fn fill_fn_params(
    ctx: &CompletionContext<'_>,
    function: &ast::Fn,
    param_list: &ast::ParamList,
    current_param: &ast::Param,
    impl_or_trait: &Option<Either<ast::Impl, ast::Trait>>,
    mut add_new_item_to_acc: impl FnMut(&str),
) {
    let mut file_params = FxHashMap::default();

    let mut extract_params = |f: ast::Fn| {
        if !is_simple_param(current_param) {
            return;
        }
        f.param_list().into_iter().flat_map(|it| it.params()).for_each(|param| {
            if let Some(pat) = param.pat() {
                let whole_param = param.to_smolstr();
                let binding = pat.to_smolstr();
                file_params.entry(whole_param).or_insert(binding);
            }
        });
    };

    for node in ctx.token.parent_ancestors() {
        match_ast! {
            match node {
                ast::SourceFile(it) => it.items().filter_map(|item| match item {
                    ast::Item::Fn(it) => Some(it),
                    _ => None,
                }).for_each(&mut extract_params),
                ast::ItemList(it) => it.items().filter_map(|item| match item {
                    ast::Item::Fn(it) => Some(it),
                    _ => None,
                }).for_each(&mut extract_params),
                ast::AssocItemList(it) => it.assoc_items().filter_map(|item| match item {
                    ast::AssocItem::Fn(it) => Some(it),
                    _ => None,
                }).for_each(&mut extract_params),
                _ => continue,
            }
        };
    }

    if let Some(stmt_list) = function.syntax().parent().and_then(ast::StmtList::cast)
        && is_simple_param(current_param)
    {
        params_from_stmt_list_scope(ctx, stmt_list, |name, ty| {
            file_params
                .entry(format_smolstr!("{}: {ty}", name.display(ctx.db, ctx.edition)))
                .or_insert(name.display(ctx.db, ctx.edition).to_smolstr());
        });
    }
    remove_duplicated(&mut file_params, param_list.params());
    let self_completion_items = ["self", "&self", "mut self", "&mut self"];
    if should_add_self_completions(ctx.token.text_range().start(), param_list, impl_or_trait) {
        self_completion_items.into_iter().for_each(&mut add_new_item_to_acc);
    }

    file_params.keys().for_each(|whole_param| add_new_item_to_acc(whole_param));
}

fn params_from_stmt_list_scope(
    ctx: &CompletionContext<'_>,
    stmt_list: ast::StmtList,
    mut cb: impl FnMut(hir::Name, String),
) {
    let syntax_node = match stmt_list.syntax().last_child() {
        Some(it) => it,
        None => return,
    };
    if let Some(scope) =
        ctx.sema.scope_at_offset(stmt_list.syntax(), syntax_node.text_range().end())
    {
        let module = scope.module().into();
        scope.process_all_names(&mut |name, def| {
            if let hir::ScopeDef::Local(local) = def
                && let Ok(ty) = local.ty(ctx.db).display_source_code(ctx.db, module, true)
            {
                cb(name, ty);
            }
        });
    }
}

fn remove_duplicated(
    file_params: &mut FxHashMap<SmolStr, SmolStr>,
    fn_params: ast::AstChildren<ast::Param>,
) {
    fn_params.for_each(|param| {
        let whole_param = param.to_smolstr();
        file_params.remove(&whole_param);

        match param.pat() {
            // remove suggestions for patterns that already exist
            // if the type is missing we are checking the current param to be completed
            // in which case this would find itself removing the suggestions due to itself
            Some(pattern) if param.ty().is_some() => {
                let binding = pattern.to_smolstr();
                file_params.retain(|_, v| v != &binding);
            }
            _ => (),
        }
    })
}

fn should_add_self_completions(
    cursor: TextSize,
    param_list: &ast::ParamList,
    impl_or_trait: &Option<Either<ast::Impl, ast::Trait>>,
) -> bool {
    if impl_or_trait.is_none() || param_list.self_param().is_some() {
        return false;
    }
    match param_list.params().next() {
        Some(first) => first.pat().is_some_and(|pat| pat.syntax().text_range().contains(cursor)),
        None => true,
    }
}

fn comma_wrapper(ctx: &CompletionContext<'_>) -> Option<(impl Fn(&str) -> SmolStr, TextRange)> {
    let param =
        ctx.original_token.parent_ancestors().find(|node| node.kind() == SyntaxKind::PARAM)?;

    let next_token_kind = {
        let t = param.last_token()?.next_token()?;
        let t = algo::skip_whitespace_token(t, Direction::Next)?;
        t.kind()
    };
    let prev_token_kind = {
        let t = param.first_token()?.prev_token()?;
        let t = algo::skip_whitespace_token(t, Direction::Prev)?;
        t.kind()
    };

    let has_trailing_comma =
        matches!(next_token_kind, SyntaxKind::COMMA | SyntaxKind::R_PAREN | SyntaxKind::PIPE);
    let trailing = if has_trailing_comma { "" } else { "," };

    let has_leading_comma =
        matches!(prev_token_kind, SyntaxKind::COMMA | SyntaxKind::L_PAREN | SyntaxKind::PIPE);
    let leading = if has_leading_comma { "" } else { ", " };

    Some((move |label: &_| format_smolstr!("{leading}{label}{trailing}"), param.text_range()))
}

fn is_simple_param(param: &ast::Param) -> bool {
    param
        .pat()
        .is_none_or(|pat| matches!(pat, ast::Pat::IdentPat(ident_pat) if ident_pat.pat().is_none()))
}

/// When completing inside a closure's param list that is an argument to a call,
/// suggest parameter lists based on the expected `Fn*` trait signature.
///
/// E.g. for `foo(|$0|)` where `foo` expects `impl Fn(usize, String) -> bool`,
/// suggest `a, b` and `a: usize, b: String`.
pub(crate) fn complete_closure_within_param(
    acc: &mut Completions,
    ctx: &CompletionContext<'_>,
) -> Option<()> {
    // Walk up: PARAM_LIST -> CLOSURE_EXPR -> ARG_LIST -> CALL_EXPR/METHOD_CALL_EXPR
    let closure_param_list = ctx.token.parent().filter(|n| n.kind() == SyntaxKind::PARAM_LIST)?;
    let closure = closure_param_list.parent().filter(|n| n.kind() == SyntaxKind::CLOSURE_EXPR)?;
    let arg_list = closure.parent().filter(|n| n.kind() == SyntaxKind::ARG_LIST)?;
    _ = arg_list
        .parent()
        .filter(|n| matches!(n.kind(), SyntaxKind::CALL_EXPR | SyntaxKind::METHOD_CALL_EXPR))?;

    let (callable, index) = callable_for_token(&ctx.sema, closure.first_token()?)?;
    let index = index?;

    // We must look at the *generic* function definition's param type, not the
    // instantiated one from the callable. When the closure is just `|`, inference
    // yields `{unknown}` for the instantiated type. The generic param type
    // (e.g. `impl Fn(usize) -> u32`) lives in the function's param env, so
    // `as_callable` can resolve the Fn trait bound from there.
    let hir::CallableKind::Function(fun) = callable.kind() else {
        return None;
    };
    // Use the absolute index (which includes self) to index into assoc_fn_params,
    // so that method calls with self don't cause an off-by-one.
    let abs_index = callable.params().into_iter().nth(index)?.index();
    let generic_param_ty = fun.assoc_fn_params(ctx.db).into_iter().nth(abs_index)?.ty().clone();

    if !generic_param_ty.impls_fnonce(ctx.db) {
        return None;
    }

    let fn_callable = generic_param_ty.as_callable(ctx.db)?;
    let closure_params = fn_callable.params();

    // Build a set of generic param names that have already been resolved
    // (via turbofish or inference from other arguments). If a substituted
    // type is concrete (not unknown), the corresponding param is resolved.
    let resolved_param_names: FxHashSet<_> = callable
        .substitution(ctx.db)
        .map(|subst| {
            subst
                .types(ctx.db)
                .into_iter()
                .filter(|(_, ty)| !ty.contains_unknown())
                .map(|(name, _)| name)
                .collect()
        })
        .unwrap_or_default();

    let module = ctx.scope.module().into();
    let source_range = ctx.source_range();
    let cap = ctx.config.snippet_cap;

    // For each closure param, include a type annotation only if the type
    // contains generic type parameters (meaning inference alone can't determine it)
    // AND the instantiated type hasn't already resolved them.
    let mut label = String::from("|");
    let mut snippet = String::new();
    let mut plain = String::new();
    let mut tab_stop = 1;

    for (i, p) in closure_params.iter().enumerate() {
        let sep = if i > 0 { ", " } else { "" };
        let ty = p.ty();
        // A type annotation is needed only if the type contains generic params
        // that haven't been resolved by the calling context.
        let needs_annotation = ty.generic_params(ctx.db).iter().any(|gp| {
            let name = gp.name(ctx.db);
            !resolved_param_names.contains(name.symbol())
        });

        if needs_annotation {
            if let Ok(ty_str) = ty.display_source_code(ctx.db, module, true) {
                write!(label, "{sep}_: {ty_str}").unwrap();
                write!(snippet, "{sep}${{{tab_stop}:_}}: ${{{}:{ty_str}}}", tab_stop + 1).unwrap();
                write!(plain, "{sep}_: {ty_str}").unwrap();
                tab_stop += 2;
            } else {
                write!(label, "{sep}_").unwrap();
                write!(snippet, "{sep}${{{tab_stop}:_}}").unwrap();
                write!(plain, "{sep}_").unwrap();
                tab_stop += 1;
            }
        } else {
            write!(label, "{sep}_").unwrap();
            write!(snippet, "{sep}${{{tab_stop}:_}}").unwrap();
            write!(plain, "{sep}_").unwrap();
            tab_stop += 1;
        }
    }

    label.push_str("| ");
    snippet.push_str("| $0");
    plain.push_str("| ");

    let mut item =
        CompletionItem::new(CompletionItemKind::Binding, source_range, &label, ctx.edition);
    match cap {
        Some(cap) => item.insert_snippet(cap, &snippet),
        None => item.insert_text(&plain),
    };
    item.add_to(acc, ctx.db);

    Some(())
}
