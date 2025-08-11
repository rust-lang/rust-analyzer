use either::Either::{self, Left, Right};
use ide_db::assists::AssistId;
use itertools::Itertools;
use syntax::NodeOrToken::{Node, Token};
use syntax::SyntaxNode;
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
    let cfg = ctx.find_node_at_offset::<ast::Attr>()?;
    let target = cfg.syntax().text_range();
    let (origin, cfg, stmt) = find_stmt(cfg)?;

    if !is_cfg(&cfg) {
        return None;
    }

    let cfg_tt = cfg.token_tree()?;
    let cfg_expr = make::expr_macro(cfg.path()?, cfg_tt.clone()).into();

    acc.add(
        AssistId::refactor_rewrite("convert_attr_cfg_to_if"),
        "Convert `#[cfg()]` to `if cfg!()`",
        target,
        |builder| {
            let mut edit = builder.make_editor(origin.syntax());

            let block = process_stmt(&cfg, stmt, &mut edit);
            let else_branch = take_else_branch(&origin, cfg_tt, &mut edit);
            let if_expr = make::expr_if(cfg_expr, block, else_branch).clone_for_update();

            if_expr.indent(origin.indent_level());
            edit.replace(origin.syntax(), if_expr.syntax());

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn take_else_branch(
    origin: &impl AstNode,
    cfg_tt: ast::TokenTree,
    edit: &mut SyntaxEditor,
) -> Option<ast::ElseBranch> {
    let attr_cfg_not = next_attr_cfg_not(origin, &cfg_tt)?;
    let (else_origin, else_cfg, else_stmt) = find_stmt(attr_cfg_not)?;
    let block = process_stmt(&else_cfg, else_stmt, edit);

    remove_next_ws(origin.syntax(), edit);
    edit.delete(else_origin.syntax());

    Some(ast::ElseBranch::Block(block))
}

fn make_block(
    stmts: impl IntoIterator<Item = ast::Stmt>,
    tail_expr: Option<ast::Expr>,
) -> ast::BlockExpr {
    let tail_expr = tail_expr.inspect(|expr| expr.reindent_to(1.into()));
    let stmts = stmts.into_iter().inspect(|stmt| stmt.reindent_to(1.into()));

    make::block_expr(stmts, tail_expr)
}

fn process_stmt(
    cfg: &ast::Attr,
    stmt: Either<ast::StmtList, ast::Stmt>,
    edit: &mut SyntaxEditor,
) -> ast::BlockExpr {
    match stmt {
        Left(stmt_list) => {
            remove_cfg(cfg, edit);
            make_block(stmt_list.statements(), stmt_list.tail_expr())
        }
        Right(stmt) => make_block([stmt], None),
    }
}

fn find_stmt(
    attr: ast::Attr,
) -> Option<(impl Indent, ast::Attr, Either<ast::StmtList, ast::Stmt>)> {
    if let Some(stmt_list) = find_stmt_list(&attr) {
        let new_stmt_list = stmt_list.clone_for_update();
        new_stmt_list.dedent(stmt_list.indent_level());
        return Some((Left(stmt_list), attr.clone(), Left(new_stmt_list)));
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

fn next_attr_cfg_not(node: &impl AstNode, cond: &ast::TokenTree) -> Option<ast::Attr> {
    let attr = node
        .syntax()
        .ancestors()
        .filter_map(|node| node.next_sibling())
        .flat_map(|x| x.descendants())
        .filter_map(ast::Attr::cast)
        .find(is_cfg)?;

    let tts = attr
        .token_tree()?
        .token_trees_and_tokens()
        .filter(|tt| tt.as_token().is_none_or(|t| !t.kind().is_trivia()))
        .collect_array()?;
    if let [Token(_lparen), Token(not), Node(not_cond), Token(_rparen)] = tts
        && not.text() == "not"
        && not_cond.syntax().text() == cond.syntax().text()
    {
        Some(attr)
    } else {
        None
    }
}

fn remove_cfg(cfg: &ast::Attr, edit: &mut SyntaxEditor) {
    remove_next_ws(cfg.syntax(), edit);
    edit.delete(cfg.syntax());
}

fn remove_next_ws(node: &SyntaxNode, edit: &mut SyntaxEditor) {
    if let Some(Token(next)) = node.next_sibling_or_token()
        && next.kind() == SyntaxKind::WHITESPACE
    {
        edit.delete(next);
    } else if let Some(parent) = node.parent()
        && parent.last_child().is_some_and(|it| it == *node)
    {
        remove_next_ws(&parent, edit);
    }
}

fn is_cfg(attr: &ast::Attr) -> bool {
    attr.path().and_then(|p| p.as_single_name_ref()).is_some_and(|name| name.text() == "cfg")
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
    fn test_stmt_list_else() {
        check_assist(
            convert_attr_cfg_to_if,
            r#"
fn foo() {
    $0#[cfg(feature = "foo")]
    {
        let x = 2;
        let _ = x+1;
    }
    #[cfg(not(feature = "foo"))]
    {
        let _ = 3;
    }
    // needless comment
}
            "#,
            r#"
fn foo() {
    if cfg!(feature = "foo") {
        let x = 2;
        let _ = x+1;
    } else {
        let _ = 3;
    }
    // needless comment
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

        check_assist(
            convert_attr_cfg_to_if,
            r#"
fn bar() {}
fn baz() {}
fn foo() {
    #[allow(unused)]
    $0#[cfg(feature = "foo")]
    bar();
    #[allow(unused)]
    #[cfg(not(feature = "foo"))]
    baz();
}
            "#,
            r#"
fn bar() {}
fn baz() {}
fn foo() {
    if cfg!(feature = "foo") {
        #[allow(unused)]
        bar();
    } else {
        #[allow(unused)]
        baz();
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
        #[cfg(not(feature = "foo"))]
        {
            let _ = match () {
                () => {
                    todo!("")
                },
            };
            match () {
                () => {
                    todo!("")
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
        } else {
            let _ = match () {
                () => {
                    todo!("")
                },
            };
            match () {
                () => {
                    todo!("")
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
        #[cfg(not(feature = "foo"))]
        match () {
            () => {
                todo!("")
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
        } else {
            match () {
                () => {
                    todo!("")
                },
            };
        }
    }
}
            "#,
        );
    }
}
