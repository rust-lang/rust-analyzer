use syntax::{
    AstNode, SyntaxKind, SyntaxNode, T,
    ast::{self, edit::AstNodeEdit, make},
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: split_if
//
// Split if condition `&&` and `||`.
//
// ```
// fn foo() {
//     if a $0&& b && c {
//         todo!()
//     }
// }
// ```
// ->
// ```
// fn foo() {
//     if a {
//         if b && c {
//             todo!()
//         }
//     }
// }
// ```
// ---
// ```
// fn foo() {
//     if a $0|| b || c {
//         todo!()
//     }
// }
// ```
// ->
// ```
// fn foo() {
//     if a {
//         todo!()
//     } else if b || c {
//         todo!()
//     }
// }
// ```
pub(crate) fn split_if(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let expr = ctx.find_node_at_offset::<ast::BinExpr>()?;
    let oper = expr.op_token()?;
    let if_ = expr.syntax().ancestors().find_map(ast::IfExpr::cast)?;
    let cond = if_.condition()?;
    let block = if_.then_branch()?;
    let (lhs, rhs) = expr.lhs().zip(expr.rhs())?;

    if !matches!(oper.kind(), T![&&] | T![||]) {
        return None;
    }
    if !is_splitable(expr.syntax(), cond.syntax(), oper.kind()) {
        return None;
    }
    if !oper.text_range().contains(ctx.offset()) {
        return None;
    }

    let new_if = if_.clone_subtree();
    let mut if_edit = SyntaxEditor::new(new_if.syntax().clone());
    let new_if_offset = if_.syntax().text_range().start();
    let new_cond = new_if.condition()?.syntax().clone();
    let new_expr = new_if.syntax().covering_element(expr.syntax().text_range() - new_if_offset);

    acc.add(
        AssistId::refactor_rewrite("split_if"),
        "Split if",
        expr.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(if_.syntax());

            if oper.kind() == T![||] {
                if_edit.replace(new_cond, lhs.syntax().clone_for_update());
                let _ = remove_else_branch(&new_if, &mut if_edit);
                edit.insert_all(
                    Position::before(if_.syntax()),
                    vec![
                        if_edit.finish().new_root().clone().into(),
                        make::tokens::single_space().into(),
                        make::token(T![else]).into(),
                        make::tokens::single_space().into(),
                    ],
                );
                edit.replace(expr.syntax(), rhs.syntax());
            } else {
                if_edit.replace(new_expr, rhs.syntax().clone_for_update());
                let new_if = if_edit.finish().new_root().clone();
                if let Some(new_if) = ast::Expr::cast(new_if) {
                    let new_block = make::block_expr(None, Some(new_if));
                    edit.replace(block.syntax(), new_block.indent(1.into()).syntax());
                }
                edit.replace(cond.syntax(), lhs.syntax());
            }

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn remove_else_branch(new_if: &ast::IfExpr, if_edit: &mut SyntaxEditor) -> Option<()> {
    let else_token = new_if.else_token()?;

    if_edit.delete(&else_token);

    if else_token.prev_token()?.kind() == SyntaxKind::WHITESPACE {
        if_edit.delete(else_token.prev_token()?);
    }
    if else_token.next_token()?.kind() == SyntaxKind::WHITESPACE {
        if_edit.delete(else_token.next_token()?);
    }
    if_edit.delete(new_if.else_branch()?.syntax());

    Some(())
}

fn is_splitable(expr: &SyntaxNode, cond: &SyntaxNode, kind: SyntaxKind) -> bool {
    if expr == cond {
        return true;
    }
    if let Some(parent) = expr.parent()
        && let Some(bin_expr) = ast::BinExpr::cast(parent)
        && !(kind == T![&&] && bin_expr.op_token().is_some_and(|it| it.kind() == T![||]))
    {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_split_if_and() {
        check_assist(
            split_if,
            r#"
fn foo() {
    if a $0&& b && c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
            r#"
fn foo() {
    if a {
        if b && c {
            let _x = [
                1,
                2,
            ];
        }
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_and_with_else_branch() {
        check_assist(
            split_if,
            r#"
fn foo() {
    if a $0&& b && c {
        let _x = [
            1,
            2,
        ];
    } else {
        foo()
    }
}
            "#,
            r#"
fn foo() {
    if a {
        if b && c {
            let _x = [
                1,
                2,
            ];
        } else {
            foo()
        }
    } else {
        foo()
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_and_in_else_branch() {
        check_assist(
            split_if,
            r#"
fn foo() {
    if x {
        todo!()
    } else if a $0&& b && c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
            r#"
fn foo() {
    if x {
        todo!()
    } else if a {
        if b && c {
            let _x = [
                1,
                2,
            ];
        }
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_or() {
        check_assist(
            split_if,
            r#"
fn foo() {
    if a $0|| b || c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
            r#"
fn foo() {
    if a {
        let _x = [
            1,
            2,
        ];
    } else if b || c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
        );

        check_assist(
            split_if,
            r#"
fn foo() {
    if a || b $0|| c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
            r#"
fn foo() {
    if a || b {
        let _x = [
            1,
            2,
        ];
    } else if c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_or_with_else_branch() {
        check_assist(
            split_if,
            r#"
fn foo() {
    if a $0|| b || c {
        let _x = [
            1,
            2,
        ];
    } else {
        foo()
    }
}
            "#,
            r#"
fn foo() {
    if a {
        let _x = [
            1,
            2,
        ];
    } else if b || c {
        let _x = [
            1,
            2,
        ];
    } else {
        foo()
    }
}
            "#,
        );

        check_assist(
            split_if,
            r#"
fn foo() {
    if a $0|| b || c {
        let _x = [
            1,
            2,
        ];
    }else{
        foo()
    }
}
            "#,
            r#"
fn foo() {
    if a {
        let _x = [
            1,
            2,
        ];
    } else if b || c {
        let _x = [
            1,
            2,
        ];
    }else{
        foo()
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_or_in_else_branch() {
        check_assist(
            split_if,
            r#"
fn foo() {
    if x {
        todo!()
    } else if a $0|| b || c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
            r#"
fn foo() {
    if x {
        todo!()
    } else if a {
        let _x = [
            1,
            2,
        ];
    } else if b || c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_not_applicable_without_operator() {
        check_assist_not_applicable(
            split_if,
            r#"
fn foo() {
    if $0a || b || c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_not_applicable_and_in_or() {
        check_assist_not_applicable(
            split_if,
            r#"
fn foo() {
    if a $0&& b || c {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
        );
    }

    #[test]
    fn test_split_if_not_applicable_in_other_expr() {
        check_assist_not_applicable(
            split_if,
            r#"
fn foo() {
    if (a $0&& b) {
        let _x = [
            1,
            2,
        ];
    }
}
            "#,
        );
    }
}
