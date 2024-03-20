use crate::{AssistContext, AssistId, AssistKind, Assists};

use syntax::{
    ast::{self,  vst::*},
     AstNode, 
};

// this proof acion introduces failing precondition in the context of the callsite
// With the IDE integration, proof action context saves the failing assertion/requires/ensures when it runs Verus 
// this proof action uses the saved verification error info.

pub(crate) fn intro_failing_requires(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // setup basic variables
    let call: ast::CallExpr = ctx.find_node_at_offset()?;
    let v_call = CallExpr::try_from(call.clone()).ok()?;
    let result = vst_rewriter_intro_failing_requires(ctx, v_call.clone())?;
    let result = ctx.fmt(call.clone(),result.to_string())?;

    acc.add(
        AssistId("intro_failing_requires", AssistKind::RefactorRewrite),
        "Insert failing requires clauses of this function call",
        call.syntax().text_range(),
        |edit| {
            edit.replace(call.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_intro_failing_requires(
    ctx: &AssistContext<'_>,
    call: CallExpr,
) -> Option<BlockExpr> {
    let name_ref = ctx.name_ref_from_call_expr(&call)?;
    let func = ctx.vst_find_fn(&call)?;
    let pre_fails = ctx.pre_failures_by_calling_this_fn(&func)?;
    let failed_exprs: Option<Vec<Expr>> = pre_fails.into_iter().map(|p| ctx.expr_from_pre_failure(p)).collect(); 
    let failed_exprs = failed_exprs?;
    let requires: Option<Vec<Expr>> = failed_exprs
                            .into_iter()
                            .map(|e| ctx.vst_inline_call(name_ref.clone(), e))
                            .collect();
    let requires = requires?;
    let mut asserts_failed_exprs: Vec<Stmt> = requires.into_iter().map(|e| {
        AssertExpr::new(e).into()}).collect::<Vec<Stmt>>();
    asserts_failed_exprs.push(call.into());
    let mut stmts = StmtList::new();
    stmts.statements = asserts_failed_exprs;
    let blk = BlockExpr::new(stmts);
    return Some(blk);
}
 

#[cfg(test)]
mod tests {
    use syntax::TextRange;

    use crate::{tests::{check_assist, check_assist_with_verus_error}, verus_error::{self, PreFailure, VerusError}};

    fn mk_pre_failure(pre_start: u32, pre_end: u32, call_start: u32, call_end: u32) -> VerusError {
        verus_error::VerusError::Pre(PreFailure{ failing_pre: TextRange::new(pre_start.into(),pre_end.into()) , callsite: TextRange::new(call_start.into(), call_end.into())})
    }

    use super::*;

    #[test] #[ignore = "need a test infra for saved verus error info"]
    fn intro_requires_easy() {
        check_assist(
            intro_failing_requires,
            r#"
proof fn my_proof_fun(x: u32, y: u32)
    requires
        x > 0,
        y > 0,
    ensures
        x * y > 0,
{       
    let multiplied = x * y;
}

proof fn call_fun(a: u32, b: u32)
    requires
        a > 0,
        b > 0,
    ensures
        a * b > 0,
{
    my_proof_fun$0(a, b);
}
"#,
            r#"
proof fn my_proof_fun(x: u32, y: u32)
    requires
        x > 0,
        y > 0,
    ensures
        x * y > 0,
{       
    let multiplied = x * y;
}

proof fn call_fun(a: u32, b: u32)
    requires
        a > 0,
        b > 0,
    ensures
        a * b > 0,
{
    assert(a > 0);
    assert(b > 0);
    my_proof_fun(a, b);
}
"#,
        );
    }

    #[test] #[ignore = "need a test infra for saved verus error info"]
    fn intro_requires_recursive() {
        check_assist(
            intro_failing_requires,
            r#"
spec fn fibo(n: nat) -> nat
    decreases n
{
    if n == 0 { 0 } else if n == 1 { 1 }
    else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}

proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires
        i <= j,
    ensures
        fibo(i) <= fibo(j),
    decreases j - i
{
    if i < 2 && j < 2 {
    } else if i == j {
    } else if i == j - 1 {
        reveal_with_fuel(fibo, 2);
        lemma_fibo_is_monotonic$0(i, (j - 1) as nat);
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
    }
}   
"#,
            r#"
spec fn fibo(n: nat) -> nat
    decreases n
{
    if n == 0 { 0 } else if n == 1 { 1 }
    else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}

proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires
        i <= j,
    ensures
        fibo(i) <= fibo(j),
    decreases j - i
{
    if i < 2 && j < 2 {
    } else if i == j {
    } else if i == j - 1 {
        reveal_with_fuel(fibo, 2);
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
    }
}   
            


"#,
        );
    }

    #[test]
    fn intro_requires_mul_ineq() {
        check_assist_with_verus_error(
            intro_failing_requires,
            vec![mk_pre_failure(87, 102, 332, 372)], 
            // `x <= y && z > 0` is at offset (87, 102) 
            // `lemm$0a_mul_inequality(x, xbound - 1, y)` is at offset (332, 372)
            r#"
proof fn lemma_mul_inequality(x: int, y: int, z: int) by(nonlinear_arith)
    requires x <= y && z > 0
    ensures  x * z <= y * z    
{}

proof fn lemma_mul_strict_upper_bound(x: int, xbound: int, y: int, ybound: int)
    requires x < xbound && y < ybound && 0 <= x && 0 <= y
    ensures x * y <= (xbound - 1) * (ybound - 1)
{
    lemm$0a_mul_inequality(x, xbound - 1, y);
    lemma_mul_inequality(y, ybound-1, xbound-1);
}
"#,
            r#"
proof fn lemma_mul_inequality(x: int, y: int, z: int) by(nonlinear_arith)
    requires x <= y && z > 0
    ensures  x * z <= y * z    
{}

proof fn lemma_mul_strict_upper_bound(x: int, xbound: int, y: int, ybound: int)
    requires x < xbound && y < ybound && 0 <= x && 0 <= y
    ensures x * y <= (xbound - 1) * (ybound - 1)
{
    {
        assert(x <= xbound - 1 && y > 0);
        lemma_mul_inequality(x, xbound - 1, y);
    };
    lemma_mul_inequality(y, ybound-1, xbound-1);
}
"#,
        );
    }


}
