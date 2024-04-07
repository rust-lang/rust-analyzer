use crate::{assist_context::{AssistContext, Assists}, AssistId, AssistKind};
use syntax::{ast::{self, vst::*, AstNode},T,};

// return `None` when this proof action is not applicable
pub(crate) fn assert_by(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on "assert"
    let _ = ctx.at_this_token(T![assert])?;

    // retrieve the assertion of interest
    let expr: ast::AssertExpr = ctx.find_node_at_offset()?;

    // lift CST into TOST node
    let assert: AssertExpr = AssertExpr::try_from(expr.clone()).ok()?;

    // edit TOST node
    let result = rewriter_assert_by(assert.clone())?;

    // pretty-print
    let result = ctx.fmt(expr.clone(),result.to_string())?;
    
    // register proof action
    acc.add(
        AssistId("assert_by", AssistKind::RefactorRewrite),
        "Add proof block for this assert",
        expr.syntax().text_range(),
        |edit| {
            edit.delete(expr.syntax().text_range());
            edit.insert(expr.syntax().text_range().start(), result);
        },
    )
}

// this function does the rewrite
pub(crate) fn rewriter_assert_by(mut assert: AssertExpr) -> Option<AssertExpr> {
    // if it already has a "by block", report "not applicable" by returning None
    if assert.by_token {
        return None;
    }
    
    // generate an empty proof block and put the same assertion in it
    let mut stmt = StmtList::new();
    stmt.statements.push(assert.clone().into());
    let blk_expr: BlockExpr = BlockExpr::new(stmt);

    // register the above proof block as our assertion's proof block
    assert.block_expr = Some(Box::new(blk_expr));
    assert.by_token = true; 
    Some(assert)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_assert_by() {
        check_assist(
            assert_by, // the proof action to be used
// proof to be modified below
// `$0` indicates the cursor location
            "
proof fn f() { 
    ass$0ert(x == 3);
}
            ",
// modified proof below
            "
proof fn f() { 
    assert(x == 3) by {
        assert(x == 3);
    };
}
            ",
        )
    }

    #[test]
    fn test_assert_by2() {
        check_assist(
            assert_by, // the proof action to be used
// proof to be modified below
// `$0` indicates the cursor location
            "
spec fn pow2(e: nat) -> nat 
    decreases(e),
{
    if e == 0 { 1 } else { 2 * pow2((e - 1) as nat)}
}

proof fn lemma_pow2_unfold3(e: nat) 
    requires e > 3,
    ensures pow2(e) == pow2((e-3) as nat) * 8,
{
    asse$0rt(pow2(e) == pow2((e - 3) as nat) * 8);
}
",
// modified proof below
            "
spec fn pow2(e: nat) -> nat 
    decreases(e),
{
    if e == 0 { 1 } else { 2 * pow2((e - 1) as nat)}
}

proof fn lemma_pow2_unfold3(e: nat) 
    requires e > 3,
    ensures pow2(e) == pow2((e-3) as nat) * 8,
{
    assert(pow2(e) == pow2((e - 3) as nat) * 8) by {
        assert(pow2(e) == pow2((e - 3) as nat) * 8);
    };
}
",
        )
    }
}
