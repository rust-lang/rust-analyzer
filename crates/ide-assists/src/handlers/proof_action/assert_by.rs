use crate::{assist_context::{AssistContext, Assists}, AssistId, AssistKind};
use syntax::{ast::{self, vst::*, AstNode},T,};

pub(crate) fn assert_by(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on "assert"
    let _ = ctx.at_this_token(T![assert])?;

    let expr: ast::AssertExpr = ctx.find_node_at_offset()?;
    let assert: AssertExpr = AssertExpr::try_from(expr.clone()).ok()?;
    let result = vst_rewriter_assert_by(assert.clone())?; // TODO: verusfmt
    dbg!(&result.to_string());


    // verusfmt
    // assume an fn is including the lightbulb
    // let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?.clone();
    let result = ctx.fmt( expr.clone(),result.to_string())?;
    
    
    acc.add(
        AssistId("assert_by", AssistKind::RefactorRewrite),
        "Add proof block for this assert",
        expr.syntax().text_range(),
        |edit| {
            edit.delete(expr.syntax().text_range());
            edit.insert(expr.syntax().text_range().start(), format!("\n{}", result));
            // edit.insert(expr.syntax().text_range().end(), result)
        },
    )
}

pub(crate) fn vst_rewriter_assert_by(mut assert: AssertExpr) -> Option<AssertExpr> {
    // if is already has a "by block", return None
    if assert.by_token {
        return None;
    }
    
    // generate empty proof block and put the same assertion in it
    let mut stmt = StmtList::new();
    stmt.statements.push(assert.clone().into());
    let blk_expr: BlockExpr = BlockExpr::new(stmt);
    assert.block_expr = Some(Box::new(blk_expr));
    assert.by_token = true; 
    Some(assert)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn assert_by_composite_condition() {
        check_assist(
            assert_by,
            "
proof fn f() { 
    ass$0ert(x == 3);
}
            ",
            "
proof fn f() { 
    
    assert(x == 3) by {
        assert(x == 3);
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
