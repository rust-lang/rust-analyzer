use itertools::Itertools;
use syntax::{
    AstNode, SyntaxKind,
    ast::{
        self,
        edit::{AstNodeEdit, IndentLevel},
        make,
    },
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{
    AssistId,
    assist_context::{AssistContext, Assists},
};

// Assist: convert_match_to_let
//
// Convert single arm match to let.
//
// ```
// struct Example { v: i32 }
//
// fn foo(e: Example) {
//     match e {
//         $0Example { v } => undefined(v),
//     }
// }
// ```
// ->
// ```
// struct Example { v: i32 }
//
// fn foo(e: Example) {
//     let Example { v } = e;
//     undefined(v)
// }
// ```
pub(crate) fn convert_match_to_let(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let match_ = ctx.find_node_at_offset::<ast::MatchExpr>()?;
    let expr = match_.expr()?;
    let [arm] = match_.match_arm_list()?.arms().collect_array()?;
    let pattern = arm.pat()?;
    let arm_expr = arm.expr()?;
    let in_expr = is_in_expr(&match_);

    if arm.guard().is_some()
        || ctx.offset() >= expr.syntax().text_range().start()
            && !pattern.syntax().text_range().contains(ctx.offset())
    {
        return None;
    }

    acc.add(
        AssistId::refactor_rewrite("convert_match_to_let"),
        "Convert match to let",
        match_.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(match_.syntax());
            let dedent_level = IndentLevel(if in_expr { 0 } else { 1 });
            let ws = &format!("\n{}", match_.indent_level() + in_expr as u8);

            let let_stmt = make::let_stmt(pattern.dedent(dedent_level), None, Some(expr));
            let (stmts, tail_expr) = stmt_and_exprs(arm_expr, dedent_level);

            let mut elements = vec![let_stmt.syntax().clone_for_update().into()];
            for stmt in stmts {
                elements.push(make::tokens::whitespace(ws).into());
                elements.push(stmt.syntax().clone().into());
            }

            if let Some(tail_expr) = tail_expr {
                elements.push(make::tokens::whitespace(ws).into());
                elements.push(tail_expr.syntax().clone().into());
            }

            if in_expr {
                let block =
                    make::block_expr(None, None).indent(match_.indent_level()).clone_subtree();
                let mut tedit = SyntaxEditor::new(block.syntax().clone());

                if let Some(stmt_list) = block.stmt_list()
                    && let Some(l_curly) = stmt_list.l_curly_token()
                {
                    tedit.insert_all(Position::after(&l_curly), elements);
                    tedit.insert(Position::after(&l_curly), make::tokens::whitespace(ws));
                }

                edit.replace(match_.syntax(), tedit.finish().new_root());
            } else {
                edit.replace_with_many(match_.syntax(), elements);
            }

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn is_in_expr(match_: &ast::MatchExpr) -> bool {
    match_.syntax().parent().is_some_and(|parent| {
        !matches!(parent.kind(), SyntaxKind::STMT_LIST | SyntaxKind::EXPR_STMT)
    })
}

fn stmt_and_exprs(
    expr: ast::Expr,
    dedent_level: IndentLevel,
) -> (Vec<ast::Stmt>, Option<ast::Expr>) {
    if let ast::Expr::BlockExpr(block) = expr.clone()
        && let Some(stmt_list) = block.stmt_list()
        && block.modifier().is_none()
        && block.label().is_none()
    {
        let stmts = stmt_list.statements().map(|stmt| stmt.dedent(dedent_level + 1)).collect();
        let tail_expr = stmt_list.tail_expr().map(|expr| expr.dedent(dedent_level + 1));
        (stmts, tail_expr)
    } else {
        (vec![], Some(expr.dedent(dedent_level)))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_convert_match_to_let() {
        check_assist(
            convert_match_to_let,
            "
fn foo() {
    $0match 2 {
        n => {
            foo;
            bar;
        }
    }
    baz;
}
            ",
            "
fn foo() {
    let n = 2;
    foo;
    bar;
    baz;
}
            ",
        );

        check_assist(
            convert_match_to_let,
            "
fn foo() {
    $0match 2 {
        n => {
            foo;
            bar;
            if true {
                ()
            }
        }
    }
    baz;
}
            ",
            "
fn foo() {
    let n = 2;
    foo;
    bar;
    if true {
        ()
    }
    baz;
}
            ",
        );
    }

    #[test]
    fn test_convert_match_to_let_in_tail_expression() {
        check_assist(
            convert_match_to_let,
            "
fn foo() {
    $0match 2 {
        n => {
            foo;
            bar;
        }
    }
}
            ",
            "
fn foo() {
    let n = 2;
    foo;
    bar;
}
            ",
        );
    }

    #[test]
    fn test_convert_match_to_let_in_expression() {
        check_assist(
            convert_match_to_let,
            "
fn foo() {
    let m = $0match 2 {
        n => {
            foo;
            bar;
        }
    };
}
            ",
            "
fn foo() {
    let m = {
        let n = 2;
        foo;
        bar;
    };
}
            ",
        );
    }

    #[test]
    fn test_convert_match_to_let_indentation() {
        check_assist(
            convert_match_to_let,
            "
fn foo() {
    $0match 2 {
        n => {
            {
                foo;
            }
            {
                bar;
            }
        }
    }
    baz;
}
            ",
            "
fn foo() {
    let n = 2;
    {
        foo;
    }
    {
        bar;
    }
    baz;
}
            ",
        );

        check_assist(
            convert_match_to_let,
            "
fn foo() {
    $0match 2 {
        n => foo(
            2
        )
    };
    baz;
}
            ",
            "
fn foo() {
    let n = 2;
    foo(
        2
    );
    baz;
}
            ",
        );

        check_assist(
            convert_match_to_let,
            "
fn foo() {
    let m = $0match 2 {
        n => {
            {
                foo;
            }
            {
                bar;
            }
        }
    };
}
            ",
            "
fn foo() {
    let m = {
        let n = 2;
        {
            foo;
        }
        {
            bar;
        }
    };
}
            ",
        );

        check_assist(
            convert_match_to_let,
            "
fn foo() {
    let m = $0match 2 {
        n => foo(
            2
        )
    };
}
            ",
            "
fn foo() {
    let m = {
        let n = 2;
        foo(
            2
        )
    };
}
            ",
        );
    }

    #[test]
    fn test_convert_match_to_let_not_applicable() {
        check_assist_not_applicable(
            convert_match_to_let,
            "
fn foo() {
    match 2 {$0
        n => {
            bar;
        }
    }
    baz;
}
            ",
        );

        check_assist_not_applicable(
            convert_match_to_let,
            "
fn foo() {
    match 2 {
        n => {
            $0bar;
        }
    }
    baz;
}
            ",
        );
    }
}
