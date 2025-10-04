use itertools::Itertools;
use syntax::{
    AstNode, T,
    ast::{self, Expr, edit::AstNodeEdit, syntax_factory::SyntaxFactory},
    syntax_editor::Element,
};

use crate::{AssistContext, AssistId, Assists};

pub(crate) fn unwrap_destructure_assign(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let eq_token = ctx.find_token_syntax_at_offset(T![=])?;
    let assign = ast::BinExpr::cast(eq_token.parent()?)?;
    let expr_stmt = ast::ExprStmt::cast(assign.syntax().parent()?)?;
    let (lhs, rhs) = (assign.lhs()?, assign.rhs()?);

    if !ctx.has_empty_selection() {
        return None;
    }

    match (lhs, rhs) {
        (Expr::TupleExpr(lhs), Expr::TupleExpr(rhs)) => {
            unwrap_tuple_destructure_assign(acc, ctx, expr_stmt, lhs, rhs)
        }
        _ => None,
    }
}

// Assist: unwrap_tuple_destructure_assign
//
// Unwrap the tuple destructuration assignment to different variables.
//
// ```
// fn foo() {
//     let (a, b);
//     (a, b) $0= (2, 3);
// }
// ```
// ->
// ```
// fn foo() {
//     let (a, b);
//     a = 2;
//     b = 3;
// }
// ```
fn unwrap_tuple_destructure_assign(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
    expr_stmt: ast::ExprStmt,
    lhs: ast::TupleExpr,
    rhs: ast::TupleExpr,
) -> Option<()> {
    if lhs.fields().count() != rhs.fields().count() {
        cov_mark::hit!(unwrap_destructure_assign_unpaired);
        return None;
    }

    acc.add(
        AssistId::refactor_rewrite("unwrap_tuple_destructure_assign"),
        "Unwrap tuple destructure assign",
        expr_stmt.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(expr_stmt.syntax());
            let make = SyntaxFactory::with_mappings();

            let indent = expr_stmt.indent_level();
            let whitespace = make.whitespace(&format!("\n{indent}")).syntax_element();
            let pairs = lhs.fields().zip(rhs.fields()).map(|(lhs, rhs)| {
                let assignment = make.expr_assignment(lhs, rhs);
                make.expr_stmt(assignment.into()).syntax().syntax_element()
            });
            let pairs = Itertools::intersperse(pairs, whitespace);

            edit.replace_with_many(expr_stmt.syntax(), pairs.collect());

            edit.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist_not_applicable;

    use super::*;

    #[test]
    fn tuple_not_applicable_not_on_eq_token() {
        check_assist_not_applicable(
            unwrap_destructure_assign,
            "fn foo() { let (a, b); (a, b) = $0(1, 2); }",
        );
        check_assist_not_applicable(
            unwrap_destructure_assign,
            "fn foo() { let (a, b); ($0a, b) = (1, 2); }",
        );
    }

    #[test]
    fn not_applicable_with_unpaired() {
        cov_mark::check!(unwrap_destructure_assign_unpaired);

        check_assist_not_applicable(
            unwrap_destructure_assign,
            "fn foo() { let (a, b); (a, b, c) $0= (1, 2); }",
        );
    }
}
