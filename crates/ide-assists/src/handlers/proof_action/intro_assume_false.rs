use crate::{assist_context::{AssistContext, Assists}, AssistId, AssistKind};
use syntax::{ast::{self, vst::*, AstNode},T,};

pub(crate) fn by_assume_false(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on "assert"
    let _ = ctx.at_this_token(T![assert])?;

    // retrieve the assertion of interest
    let expr: ast::AssertExpr = ctx.find_node_at_offset()?;

    // lift CST into TOST node
    let assert: AssertExpr = AssertExpr::try_from(expr.clone()).ok()?;

    // edit TOST node
    let result = vst_rewriter_by_assume_false(ctx, assert.clone())?;

    // pretty-print
    let result = ctx.fmt(expr.clone(),result.to_string())?;
    acc.add(
        AssistId("by_assume_false", AssistKind::RefactorRewrite),
        "Insert assume false for this assert",
        expr.syntax().text_range(),
        |edit| {
            edit.replace(expr.syntax().text_range(), result.to_string());
        },
    )
}

pub(crate) fn vst_rewriter_by_assume_false(ctx: &AssistContext<'_>, mut assert: AssertExpr) -> Option<AssertExpr> {
    // if is already has a "by block", return None
    if assert.by_token {
        return None;
    }
    assert.by_token = true;

    // generate empty proof block and put the "assume(false)" in it
    let mut stmt = StmtList::new();
    let false_: Expr = ctx.vst_expr_from_text("false")?;
    let assume_false = AssumeExpr::new(false_,);
    stmt.statements.push(assume_false.into());
    let blk_expr: BlockExpr = BlockExpr::new(stmt);
    assert.block_expr = Some(Box::new(blk_expr));
    Some(assert)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_by_assume_false1() {
        check_assist(
            by_assume_false,
            "
proof fn f(x: int) { 
    ass$0ert(x == 3); 
}
            ",
            "
proof fn f(x: int) { 
    assert(x == 3) by {
        assume(false);
    }; 
}
            ",
        )
    }
}

// let formatter = "/home/chanhee/.cargo/bin/rustfmt";
// let formatted_string = Command::new("echo")
//     .arg(string.clone())
//     .arg("|")
//     .arg(formatter)
//     .spawn()
//     .expect("echo command failed to start").stdout.unwrap();
// dbg!(formatted_string);
