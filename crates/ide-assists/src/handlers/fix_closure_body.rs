use ide_db::syntax_helpers::suggest_name::NameGenerator;
use syntax::{
    AstNode, SmolStr, SyntaxElement, T,
    ast::{self, syntax_factory::SyntaxFactory},
    syntax_editor::{Element, Position},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: fix_closure_body
//
// Fix incomplete closures.
//
// ```
// //- minicore: fn
// fn foo() -> impl Fn(i32) {
//     ||$0
// }
// ```
// ->
// ```
// fn foo() -> impl Fn(i32) {
//     |it| {$0}
// }
// ```
pub(crate) fn fix_closure_body(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let closure = ctx.find_node_at_offset::<ast::ClosureExpr>()?;
    let param_list = closure.param_list()?;
    let pipe_token = param_list.pipe_token()?;

    if closure.body().is_some() || param_list.params().next().is_some() {
        return None;
    }
    if !closure.syntax().text_range().contains_range(ctx.selection_trimmed()) {
        return None;
    }
    let insert_after_elem = closure
        .ret_type()
        .map(|it| it.syntax().syntax_element())
        .unwrap_or_else(|| param_list.syntax().syntax_element());

    // FIXME: generate async closure
    let params = expected_callable_type(ctx, &closure.clone().into())?.params();

    acc.add(
        AssistId::quick_fix("fix_closure_body"),
        "Fill incomplete closure",
        closure.syntax().text_range(),
        |builder| {
            let names = generate_names(&params, ctx);
            let mut edit = builder.make_editor(closure.syntax());
            let make = SyntaxFactory::with_mappings();

            edit.insert_all(Position::after(pipe_token), make_params(&names, &make));
            let block = make.expr_empty_block();
            edit.insert_all(
                Position::after(insert_after_elem),
                vec![make.whitespace(" ").syntax_element(), block.syntax().syntax_element()],
            );
            if let Some(cap) = ctx.config.snippet_cap
                && let Some(stmt_list) = block.stmt_list()
                && let Some(l_curly) = stmt_list.l_curly_token()
            {
                let annotation = builder.make_tabstop_after(cap);
                edit.add_annotation(l_curly, annotation);
            }

            edit.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn expected_callable_type<'db>(
    ctx: &AssistContext<'db>,
    expr: &ast::Expr,
) -> Option<hir::Callable<'db>> {
    // TODO expr_adjustments is empty
    ctx.sema.expr_adjustments(expr)?.iter().find_map(|it| it.target.as_callable(ctx.db()))
}

fn make_params(names: &[SmolStr], make: &SyntaxFactory) -> Vec<SyntaxElement> {
    names
        .iter()
        .enumerate()
        .flat_map(|(i, name)| {
            (i == 0)
                .then(|| [make.token(T![,]), make.whitespace(" ")])
                .into_iter()
                .flatten()
                .chain(Some(make.ident(&name)))
                .map(|it| it.syntax_element())
        })
        .collect()
}

fn generate_names(params: &[hir::Param<'_>], ctx: &AssistContext<'_>) -> Vec<SmolStr> {
    let mut name_generator = NameGenerator::new_with_names(std::iter::empty());
    let default_param_name = if params.len() <= 1 { "it" } else { "param" };
    params
        .iter()
        .map(|param| {
            name_generator
                .for_type(param.ty(), ctx.db(), ctx.edition())
                .unwrap_or_else(|| name_generator.suggest_name(default_param_name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn in_parameter() {
        check_assist(
            fix_closure_body,
            r#"
            //- minicore: fn
            fn foo(_: impl Fn(i32)) {}
            fn bar() {
                foo(||$0)
            }
            "#,
            r#"
            fn foo(_: impl Fn(i32)) {}
            fn bar() {
                foo(|it| {$0})
            }
            "#,
        );
    }
}
