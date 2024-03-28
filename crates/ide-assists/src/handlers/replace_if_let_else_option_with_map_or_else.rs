use ide_db::{syntax_helpers::node_ext::single_let, ty_filter::TryEnum};
use syntax::{
    ast::{self, make},
    AstNode, TextRange,
};

use crate::{utils::does_pat_match_variant, AssistContext, AssistId, AssistKind, Assists};

// Assist: replace_if_let_else_option_with_map_or_else
//
// Replaces a `if let Some(_) = ... { ... } else { ... }` expression with `Option::map_or_else`.
//
// ```
// fn do_complicated_function() -> i32 { 1 }
//
// let optional = Some(1);
//
// let _ = $0if let Some(foo) = optional {
//     foo
// } else {
//     let y = do_complicated_function();
//     y*y
// };
// ```
// ->
// ```
// fn do_complicated_function() -> i32 { 1 }
//
// let optional = Some(1);
//
// let _ = optional.map_or_else(|| {
//     let y = do_complicated_function();
//     y*y
// }, |foo| {
//     foo
// });
// ```
pub(crate) fn replace_if_let_else_option_with_map_or_else(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let if_expr: ast::IfExpr = ctx.find_node_at_offset()?;
    let available_range = TextRange::new(
        if_expr.syntax().text_range().start(),
        if_expr.then_branch()?.syntax().text_range().start(),
    );
    let cursor_in_range = available_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let Some(ast::ElseBranch::Block(else_block)) = if_expr.else_branch() else {
        return None;
    };

    let scrutinee_to_be_expr = if_expr.condition()?;
    let scrutinee_to_be_expr = match single_let(scrutinee_to_be_expr.clone()) {
        Some(cond) => cond.expr()?,
        None => scrutinee_to_be_expr,
    };
    let expr;
    let pat;
    match single_let(if_expr.clone().condition()?) {
        Some(let_) => {
            let pat_ = let_.pat()?;
            expr = let_.expr()?;
            if !does_pat_match_variant(&pat_, &TryEnum::Option.happy_pattern_wildcard()) {
                return None;
            }
            let ast::Pat::TupleStructPat(tuple_pat) = pat_ else {
                return None;
            };

            if scrutinee_to_be_expr.syntax().text() != expr.syntax().text() {
                return None;
            }
            pat = tuple_pat.fields().next()?;
        }
        None => return None,
    };
    let body = if_expr.then_branch()?;

    acc.add(
        AssistId("replace_if_let_else_option_with_map_or_else", AssistKind::RefactorRewrite),
        "Replace if let else with `Option::map_or_else`".to_owned(),
        available_range,
        move |edit| {
            let map_or_else = {
                let option_map_or_else = make::name_ref("map_or_else");
                let map_expr = make::expr_closure(Some(make::param(pat, None)), body.into());
                let else_expr = make::expr_closure(None, else_block.into());
                make::expr_method_call(
                    expr,
                    option_map_or_else,
                    make::arg_list([else_expr, map_expr]),
                )
            };
            edit.replace_ast(if_expr.into(), map_or_else)
        },
    )
}
