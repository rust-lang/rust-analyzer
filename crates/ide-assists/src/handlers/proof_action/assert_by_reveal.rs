// use ide_db::syntax_helpers::node_ext::is_pattern_cond;
use crate::{
    assist_context::{AssistContext, Assists},
    // utils::invert_boolean_expression,
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, vst::*, AstNode},
    T,
};

pub(crate) fn assert_by_reveal(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on function name
    let call: ast::CallExpr = ctx.find_node_at_offset()?;
    // if the function name is not inside an assertExpr, return None
    dbg!("get assertion");
    let assert_expr: ast::AssertExpr = ctx.find_node_at_offset()?;
    let v_call = CallExpr::try_from(call.clone()).ok()?;
    let v_assert_expr = AssertExpr::try_from(assert_expr.clone()).ok()?;
    dbg!("now rewrite vst");
    let result = vst_rewriter_assert_to_assert_by_reveal(ctx, &v_call, v_assert_expr.clone())?;


    acc.add(
        AssistId("assert_by_reveal", AssistKind::RefactorRewrite),
        "Reveal function with a new proof block for this assert",
        assert_expr.syntax().text_range(),
        |edit| {
            edit.replace(assert_expr.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_assert_to_assert_by_reveal(
    ctx: &AssistContext<'_>,
    call: &CallExpr,
    mut assert: AssertExpr,
) -> Option<String> 
{
    dbg!(&call.clone().cst?);
    // if is already has a "by block", return None
    if assert.by_token {
        return None;
    }
    // get func
    let name_ref = ctx.name_ref_from_call_expr(&call)?;
    let func = ctx.vst_find_fn(&call)?;
    // if func is not opaque, return None
    if ctx.is_opaque(&func) == false {
        return None;
    }
    
    // generate "reveal(foo)"   
    let mut arglist = ArgList::new();
    arglist.args.push(*call.expr.clone());
    let reveal_expr = ctx.vst_call_expr_from_text("reveal", arglist )?;

    // generate empty stmtlist and put "reveal(foo) in it"
    let mut stmt = StmtList::new();
    stmt.statements.push(reveal_expr.into());
    
    let blk_expr: BlockExpr = BlockExpr::new(stmt);
    assert.by_token = true;
    assert.block_expr = Some(Box::new(blk_expr));
    Some(assert.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_assert_by_reveal_1() {
        check_assist(
            assert_by_reveal,
            "
#[verifier::opaque]
spec fn opaque_fibo(n: nat) -> nat
  decreases n
{
  if n == 0 { 0 } else if n == 1 { 1 }
  else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}

proof fn test_opaque_fibo() {
  assert(opaq$0ue_fibo(2) == 1);
  }
}
",

"
#[verifier::opaque]
spec fn opaque_fibo(n: nat) -> nat
  decreases n
{
  if n == 0 { 0 } else if n == 1 { 1 }
  else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}

proof fn test_opaque_fibo() {
  assert(opaque_fibo(2) == 1) by {
    reveal(opaque_fibo);
  }
}
"
        )
    }
}
