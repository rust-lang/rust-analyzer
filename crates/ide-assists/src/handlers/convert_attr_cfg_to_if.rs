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
    let cfg = ctx.find_node_at_offset::<ast::Attr>()?;
    let stmt_list = find_stmt_list(&cfg)?;

    if cfg.path()?.as_single_name_ref()?.text() != "cfg" {
        return None;
    }

    let cfg_expr = make::expr_macro(cfg.path()?, cfg.token_tree()?).into();

    acc.add(
        AssistId::refactor_rewrite("convert_attr_cfg_to_if"),
        "Convert `#[cfg()]` to `if cfg!()`",
        cfg.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(stmt_list.syntax());

            remove_cfg(&cfg, &mut edit);

            let block = make::block_expr(stmt_list.statements(), None);
            let if_expr = make::expr_if(cfg_expr, block, None).clone_for_update();
            if_expr.indent(stmt_list.indent_level());
            edit.replace(stmt_list.syntax(), if_expr.syntax());

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
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
    }
}
