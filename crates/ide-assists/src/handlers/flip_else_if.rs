use syntax::{AstNode, T, ast, syntax_editor::SyntaxEditor};

use crate::{AssistContext, AssistId, Assists};

// Assist: flip_else_if
//
// Flips two if-else-if branches.
//
// ```
// fn foo() {
//     let n = if cond1 { 2 } else$0 if cond2 { 3 } else { 0 };
// }
// ```
// ->
// ```
// fn foo() {
//     let n = if cond2 { 3 } else if cond1 { 2 } else { 0 };
// }
// ```
pub(crate) fn flip_else_if(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let else_kw = ctx.find_token_syntax_at_offset(T![else])?;
    let if_expr = ast::IfExpr::cast(else_kw.parent()?)?;
    let ast::ElseBranch::IfExpr(else_if) = if_expr.else_branch()? else { return None };

    let (cond, block) = (if_expr.condition()?, if_expr.then_branch()?);
    let (else_cond, else_block) = (else_if.condition()?, else_if.then_branch()?);

    let target = else_kw.text_range();
    acc.add(AssistId::refactor_rewrite("flip_else_if"), "Flip if else if", target, |builder| {
        let mut edit = builder.make_editor(if_expr.syntax());

        swap_node(&cond, &else_cond, &mut edit);
        swap_node(&block, &else_block, &mut edit);

        builder.add_file_edits(ctx.vfs_file_id(), edit);
    })
}

fn swap_node(lhs: &impl AstNode, rhs: &impl AstNode, edit: &mut SyntaxEditor) {
    edit.replace(lhs.syntax(), rhs.syntax());
    edit.replace(rhs.syntax(), lhs.syntax());
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn not_applicable_without_else_keyword() {
        check_assist_not_applicable(
            flip_else_if,
            r#"
            fn foo() {
                let n = if cond1 { 2 } else if$0 cond2 { 3 } else { 0 };
            }
            "#,
        );

        check_assist_not_applicable(
            flip_else_if,
            r#"
            fn foo() {
                let n = if$0 cond1 { 2 } else if cond2 { 3 } else { 0 };
            }
            "#,
        );
    }

    #[test]
    fn without_else_branch() {
        check_assist(
            flip_else_if,
            r#"
            fn foo() {
                if cond1 { 2 } $0else if cond2 { 3 };
            }
            "#,
            r#"
            fn foo() {
                if cond2 { 3 } else if cond1 { 2 };
            }
            "#,
        );
    }

    #[test]
    fn not_applicable_else_block_branch() {
        check_assist_not_applicable(
            flip_else_if,
            r#"
            fn foo() {
                let n = if cond1 { 2 } else$0 { 3 };
            }
            "#,
        );
    }
}
