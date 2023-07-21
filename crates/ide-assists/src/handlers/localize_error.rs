use crate::{vst_api, AssistContext, Assists};
use ide_db::{
    assists::{AssistId, AssistKind},
    syntax_helpers::vst_ext::*,
};

use syntax::{
    ast::{self, vst},
    AstNode, T,
};

/*
TODO: fill in

*/

pub(crate) fn localize_error(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let stmt_list = ctx.find_node_at_offset::<ast::StmtList>()?;
    let v_stmt_list = vst::StmtList::try_from(stmt_list.clone()).ok()?;
    let result = vst_transformer_wp_move_assertion(ctx, v_stmt_list.clone())?;

    acc.add(
        AssistId("move_up_assertion", AssistKind::RefactorRewrite),
        "Move up assertion through statements ",
        stmt_list.syntax().text_range(),
        |edit| {
            edit.replace(stmt_list.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_transformer_wp_move_assertion(
    ctx: &AssistContext<'_>,
    stmt_list: vst::StmtList,
) -> Option<String> {
    let assertion = ctx.vst_find_node_at_offset::<vst::AssertExpr, ast::AssertExpr>()?;
    println!("assertion: {}", assertion);
    let index = stmt_list.statements.iter().position(|s| match s {
        vst::Stmt::ExprStmt(e) => match e.expr.as_ref() {
            vst::Expr::AssertExpr(a) => **a == assertion,
            _ => false,
        },
        _ => false,
    })?;
    if index == 0 {
        // assertion is already at the top
        return None;
    }
    None
    // insert new assertion in the right place and return
    // new_stmt_list.statements.insert(index - 1, new_stmt);
    // return Some(new_stmt_list.to_string());
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn wp_assign_easy() {
        check_assist(
            localize_error,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    assert(a > 10);
    assert(a < 100);
    assert(a > 10 && a < 100);
}
"#,
        );
    }
}
