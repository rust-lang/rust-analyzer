use crate::{AssistContext, AssistId, AssistKind, Assists};

use syntax::{
    ast::{self,  vst::*},
     AstNode, 
};

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
    dbg!(&func.name);
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
    use crate::tests::check_assist;

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
}
