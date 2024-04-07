use crate::{AssistContext, Assists};
use ide_db::assists::{AssistId, AssistKind};
use syntax::{
    ast::{self, vst::*, LogicOp},
    AstNode, T,
};

/*

Localize error by splitting assertion (cf. "--expandn-errors" in Verus)


assert(exp)

into

1) &&
assert(e1);
assert(e2);
assert(exp);

where exp = e1 && e2

2) match
if the enum has #[is_variant],
split each match arm into a separate assertion


option1:
  if let Message::Move{x, y} = x {
    assert(x > y);
  }
  if let Message::Quit(b) = y {
    assert(b);
  }
  if let Message::Write(b) = x {
    assert(b);
  }

option2:
assert(x.is_Quit() ==> x.get_Quit_0());
assert(x.is_Move() ==> x.get_Move_x() > x.get_Move_y());
assert(x.is_Write() ==> x.get_Write_0());


assert(match x {
    Message::Quit(b) => b,
    Message::Move{x, y} => x > y,
    Message::Write(b) => b,
});



*/

pub(crate) fn localize_error(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let assertion = ctx.find_node_at_offset::<ast::AssertExpr>()?;
    let v_assertion = AssertExpr::try_from(assertion.clone()).ok()?;
    let result = vst_rewriter_localize_error_minimized(ctx, v_assertion.clone())?;
    let result = ctx.fmt(assertion.clone(),result.to_string())?;

    acc.add(
        AssistId("localize_error", AssistKind::RefactorRewrite),
        "Decompose Failing Assertion",
        assertion.syntax().text_range(),
        |edit| {
            edit.replace(assertion.syntax().text_range(), result);
        },
    )
}

fn split_expr(ctx: &AssistContext<'_>, exp: &Expr) -> Option<Vec<Expr>> {
    match exp {
        Expr::BinExpr(be) => {
            match be.op {
                BinaryOp::LogicOp(LogicOp::And) => {
                    // REVIEW: instead of A and B, consider split into A and A => B
                    // Logically this makes more sense, but it might be confusing to the user
                    Some(vec![*be.lhs.clone(), *be.rhs.clone()])
                }
                BinaryOp::LogicOp(LogicOp::Imply) => {
                    let split_exprs: Vec<Expr> = split_expr(ctx, be.lhs.as_ref())?;
                    let implied_exprs: Vec<Expr> = split_exprs.into_iter().map(|e| {
                        BinExpr::new(
                            be.rhs.as_ref().clone(),
                            BinaryOp::LogicOp(LogicOp::Imply),
                            e,
                        ).into()
                    }).collect();
                    Some(implied_exprs)
                }
                _ => return None,
            }
        },
        Expr::MatchExpr(_me) => {
            // Note: #[is_variant] is now deprecated in Verus
            return None;
        },            
        Expr::CallExpr(call) => {
            dbg!("inline call for decomposing failing assertion");
            let name_ref = ctx.name_ref_from_call_expr(call)?;
            let func = ctx.vst_find_fn(&call)?;
            let spec_func_body_expr = *func.body?.clone().stmt_list.tail_expr?;
            let inlined_exp = ctx.vst_inline_call(name_ref.clone(), spec_func_body_expr)?;
            dbg!("get recursive");
            dbg!("{}", &inlined_exp.to_string());
            return split_expr(ctx, &inlined_exp);
        }
        // TODO: match, if, not, quant(forall)
        _ => return None,
    }
}

// TODO: make this a separate proof action
// pub(crate) fn _vst_rewriter_localize_error(
//     ctx: &AssistContext<'_>,
//     assertion: AssertExpr,
// ) -> Option<String> {
//     let exp = &assertion.expr;
//     let split_exprs = split_expr(ctx, exp)?;
//     let mut stmts: StmtList = StmtList::new();
//     for e in split_exprs {
//         let assert_expr = AssertExpr::new(e);
//         stmts.statements.push(assert_expr.into());
//     }
//     stmts.statements.push(assertion.into());
//     let blk = BlockExpr::new(stmts);
//     return Some(blk.to_string());
// }

// maybe register another action with minimization
pub(crate) fn vst_rewriter_localize_error_minimized(
    ctx: &AssistContext<'_>,
    assertion: AssertExpr,
) -> Option<BlockExpr> {
    let this_fn = ctx.vst_find_node_at_offset::<Fn, ast::Fn>()?; 
    let exp = &assertion.expr;
    let split_exprs = split_expr(ctx, exp)?;
    for e in &split_exprs {
        dbg!({}, e.to_string());
    }
    let mut stmts: StmtList = StmtList::new();
    for e in split_exprs {
        dbg!("{}", &e.to_string());
        let split_assert = AssertExpr::new(e);
        let modified_fn = ctx.replace_statement(&this_fn, assertion.clone(), split_assert.clone())?;
        let verif_result = ctx.try_verus(&modified_fn)?;
        if verif_result.is_failing(&split_assert) {
            dbg!(verif_result);
            // this is not enough -- need to retrieve failing assertions
            // and check if this split assertion is failing
            stmts.statements.push(split_assert.into());
        }
    }
    stmts.statements.push(assertion.into());
    let blk = BlockExpr::new(stmts);
    return Some(blk);
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;
    use super::*;

    // TEST: && (1)
    #[test]
    fn decompose_conjunct_failure() {
        check_assist(
            localize_error,
// before
            r#"
use vstd::prelude::*;
fn foo()
{
    let a:u32 = 1;
    ass$0ert(a > 10 && a < 100);
}
fn main() {}
"#,
// after
            r#"
use vstd::prelude::*;
fn foo()
{
    let a:u32 = 1;
    {
        assert(a > 10);
        assert(a > 10 && a < 100);
    };
}
fn main() {}
"#,
        );
    }


    // TEST: && (2)
    #[test]
    fn decompose_conjunct_failure2() {
        check_assist(
            localize_error,
// before
            r#"
use vstd::prelude::*;

proof fn lemma_mul_inequality(x: int, y: int, z: int) by(nonlinear_arith)
    requires x <= y && z > 0
    ensures  x * z <= y * z    
{}

proof fn lemma_mul_strict_upper_bound(x: int, xbound: int, y: int, ybound: int)
    requires x < xbound && y < ybound && 0 <= x && 0 <= y
    ensures x * y <= (xbound - 1) * (ybound - 1)
{
    {
        as$0sert(x <= xbound - 1 && y > 0);
        lemma_mul_inequality(x, xbound - 1, y);
    };
    lemma_mul_inequality(y, ybound-1, xbound-1);
}

fn main() {}
"#,
// after
            r#"
use vstd::prelude::*;

proof fn lemma_mul_inequality(x: int, y: int, z: int) by(nonlinear_arith)
    requires x <= y && z > 0
    ensures  x * z <= y * z    
{}

proof fn lemma_mul_strict_upper_bound(x: int, xbound: int, y: int, ybound: int)
    requires x < xbound && y < ybound && 0 <= x && 0 <= y
    ensures x * y <= (xbound - 1) * (ybound - 1)
{
    {
        {
            assert(y > 0);
            assert(x <= xbound - 1 && y > 0);
        };
        lemma_mul_inequality(x, xbound - 1, y);
    };
    lemma_mul_inequality(y, ybound-1, xbound-1);
}

fn main() {}
"#,
        );
    }

    // TEST: inline
    #[test]
    fn decompose_function_inline() {
        check_assist(
            localize_error,
            r#"
use vstd::prelude::*;
use vstd::seq::*;
spec fn seq_bounded_by_length(s1: Seq<int>) -> bool 
{
    forall|i:int| (0 <= i && i < s1.len())  ==>  (0 <= s1.index(i) && s1.index(i) < s1.len())
}

spec fn long_seq(s1: Seq<int>) -> bool {
    s1.len() > 20
}

spec fn seq_is_long_and_bounded_by_length(s1: Seq<int>) -> bool {
    seq_bounded_by_length(s1) && long_seq(s1)
}

proof fn test()
{
    let mut ss: Seq<int> = Seq::empty();
    ss = ss.push(0);
    ss = ss.push(1);
    as$0sert(seq_is_long_and_bounded_by_length(ss));
}

fn main() {}
"#,
            r#"
use vstd::prelude::*;
use vstd::seq::*;
spec fn seq_bounded_by_length(s1: Seq<int>) -> bool 
{
    forall|i:int| (0 <= i && i < s1.len())  ==>  (0 <= s1.index(i) && s1.index(i) < s1.len())
}

spec fn long_seq(s1: Seq<int>) -> bool {
    s1.len() > 20
}

spec fn seq_is_long_and_bounded_by_length(s1: Seq<int>) -> bool {
    seq_bounded_by_length(s1) && long_seq(s1)
}

proof fn test()
{
    let mut ss: Seq<int> = Seq::empty();
    ss = ss.push(0);
    ss = ss.push(1);
    {
        assert(long_seq(ss));
        assert(seq_is_long_and_bounded_by_length(ss));
    };
}

fn main() {}
"#,
        );
    }

}
