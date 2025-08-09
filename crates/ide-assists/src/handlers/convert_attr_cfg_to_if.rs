use either::Either::{self, Left, Right};
use ide_db::assists::AssistId;
use syntax::ast::edit_in_place::Indent;
use syntax::syntax_editor::SyntaxEditor;
use syntax::{
    AstNode, SyntaxKind,
    ast::{self, make},
};

use crate::assist_context::{AssistContext, Assists};

// Assist: convert_attr_cfg_to_if
//
// Convert `#[cfg(...)] {}` to `if cfg!(...) {}`.
//
// ```
// fn foo() {
//     $0#[cfg(feature = "foo")]
//     {
//         let _x = 2;
//     }
// }
// ```
// ->
// ```
// fn foo() {
//     if cfg!(feature = "foo") {
//         let _x = 2;
//     }
// }
// ```
pub(crate) fn convert_attr_cfg_to_if(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let (origin, cfg, stmt) = find_stmt(ctx)?;

    if cfg.path()?.as_single_name_ref()?.text() != "cfg" {
        return None;
    }

    let cfg_expr = make::expr_macro(cfg.path()?, cfg.token_tree()?).into();

    acc.add(
        AssistId::refactor_rewrite("convert_attr_cfg_to_if"),
        "Convert `#[cfg()]` to `if cfg!()`",
        cfg.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(origin.syntax());

            let block = match stmt {
                Left(stmt_list) => {
                    remove_cfg(&cfg, &mut edit);
                    make_block(stmt_list.statements(), stmt_list.tail_expr())
                }
                Right(stmt) => make_block([stmt], None),
            };
            let if_expr = make::expr_if(cfg_expr, block, None).clone_for_update();
            if_expr.indent(origin.indent_level());
            edit.replace(origin.syntax(), if_expr.syntax());

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn make_block(
    stmts: impl IntoIterator<Item = ast::Stmt>,
    tail_expr: Option<ast::Expr>,
) -> ast::BlockExpr {
    let tail_expr = tail_expr.inspect(|expr| expr.reindent_to(1.into()));
    let stmts = stmts.into_iter().inspect(|stmt| stmt.reindent_to(1.into()));

    make::block_expr(stmts, tail_expr)
}

fn find_stmt(
    ctx: &AssistContext<'_>,
) -> Option<(impl Indent, ast::Attr, Either<ast::StmtList, ast::Stmt>)> {
    let attr = ctx.find_node_at_offset::<ast::Attr>()?;

    if let Some(stmt_list) = find_stmt_list(&attr) {
        let new_stmt_list = stmt_list.clone_for_update();
        new_stmt_list.dedent(stmt_list.indent_level());
        return Some((Left(stmt_list), attr, Left(new_stmt_list)));
    }

    let node =
        match attr.syntax().ancestors().find_map(Either::<ast::ExprStmt, ast::StmtList>::cast)? {
            Left(expr_stmt) => Left(expr_stmt),
            Right(list) => Right(list.tail_expr()?),
        };

    let new_node = node.syntax().clone_subtree();
    let attr = new_node
        .descendants()
        .filter(|node| node.text() == attr.syntax().text())
        .find_map(ast::Attr::cast)?;

    let mut edit = SyntaxEditor::new(new_node);
    remove_cfg(&attr, &mut edit);
    let new_node = edit.finish().new_root().clone();
    let new_stmt = match Either::<ast::ExprStmt, ast::Expr>::cast(new_node)? {
        Left(expr_stmt) => ast::Stmt::from(expr_stmt),
        Right(expr) => make::expr_stmt(expr).clone_for_update().into(),
    };
    new_stmt.dedent(node.indent_level());
    Some((Right(node), attr, Right(new_stmt)))
}

fn find_stmt_list(attr: &ast::Attr) -> Option<ast::StmtList> {
    let mut node = attr.syntax().clone();

    while node.kind().is_trivia() || node.kind() == SyntaxKind::ATTR {
        node = node.next_sibling()?;
    }

    AstNode::cast(node)
}

fn remove_cfg(cfg: &ast::Attr, edit: &mut SyntaxEditor) {
    if let Some(next) = cfg.syntax().last_token().and_then(|it| it.next_token())
        && next.kind() == SyntaxKind::WHITESPACE
    {
        edit.delete(next);
    }
    edit.delete(cfg.syntax());
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn test_stmt_list() {
        check_assist(
            convert_attr_cfg_to_if,
            r#"
fn foo() {
    $0#[cfg(feature = "foo")]
    {
        let x = 2;
        let _ = x+1;
    }
}
            "#,
            r#"
fn foo() {
    if cfg!(feature = "foo") {
        let x = 2;
        let _ = x+1;
    }
}
            "#,
        );
    }

    #[test]
    fn test_expr_stmt() {
        check_assist(
            convert_attr_cfg_to_if,
            r#"
fn bar() {}
fn foo() {
    $0#[cfg(feature = "foo")]
    bar();
}
            "#,
            r#"
fn bar() {}
fn foo() {
    if cfg!(feature = "foo") {
        bar();
    }
}
            "#,
        );
    }

    #[test]
    fn test_other_attr() {
        check_assist(
            convert_attr_cfg_to_if,
            r#"
fn foo() {
    $0#[cfg(feature = "foo")]
    #[allow(unused)]
    {
        let x = 2;
        let _ = x+1;
    }
}
            "#,
            r#"
fn foo() {
    #[allow(unused)]
    if cfg!(feature = "foo") {
        let x = 2;
        let _ = x+1;
    }
}
            "#,
        );

        check_assist(
            convert_attr_cfg_to_if,
            r#"
fn bar() {}
fn foo() {
    #[allow(unused)]
    $0#[cfg(feature = "foo")]
    bar();
}
            "#,
            r#"
fn bar() {}
fn foo() {
    if cfg!(feature = "foo") {
        #[allow(unused)]
        bar();
    }
}
            "#,
        );
    }

    #[test]
    fn test_stmt_list_indent() {
        check_assist(
            convert_attr_cfg_to_if,
            r#"
mod a {
    fn foo() {
        #[allow(unused)]
        $0#[cfg(feature = "foo")]
        {
            let _ = match () {
                () => {
                    todo!()
                },
            };
            match () {
                () => {
                    todo!()
                },
            }
        }
    }
}
            "#,
            r#"
mod a {
    fn foo() {
        #[allow(unused)]
        if cfg!(feature = "foo") {
            let _ = match () {
                () => {
                    todo!()
                },
            };
            match () {
                () => {
                    todo!()
                },
            }
        }
    }
}
            "#,
        );
    }

    #[test]
    fn test_expr_indent() {
        check_assist(
            convert_attr_cfg_to_if,
            r#"
mod a {
    fn foo() {
        #[allow(unused)]
        $0#[cfg(feature = "foo")]
        match () {
            () => {
                todo!()
            },
        }
    }
}
            "#,
            r#"
mod a {
    fn foo() {
        if cfg!(feature = "foo") {
            #[allow(unused)]
            match () {
                () => {
                    todo!()
                },
            }
        }
    }
}
            "#,
        );

        check_assist(
            convert_attr_cfg_to_if,
            r#"
mod a {
    fn foo() {
        $0#[cfg(feature = "foo")]
        match () {
            () => {
                todo!()
            },
        };
    }
}
            "#,
            r#"
mod a {
    fn foo() {
        if cfg!(feature = "foo") {
            match () {
                () => {
                    todo!()
                },
            };
        }
    }
}
            "#,
        );
    }
}
