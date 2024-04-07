use crate::{
    assist_context::{AssistContext, Assists},
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, vst::*, AstNode, CmpOp, Ordering},
    T,
};

/// assert forall ||  A <= B ==> Q 
/// into
/// assert forall ||  A == B ==> Q 
/// assert forall ||  A <= B ==> Q 
pub(crate) fn split_smaller_or_equal_to(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on "<="
    let _ = ctx.at_this_token(T![<=])?;

    // <= is not inside an assertForallExpr, return None
    let assert_forall_expr: ast::AssertForallExpr = ctx.find_node_at_offset()?;
    // dbg!(&assert_forall_expr);


    // now convert to vst nodes
    // check fix AssertForallExpr::try_from. use  .exprs().nth(1) instead of .expr (expr gives earlier closure instead of conclusion)
    let assert = AssertForallExpr::try_from(assert_forall_expr.clone()).ok()?;    
    let conclusion = assert_forall_expr.exprs().nth(1)?;
    let v_conclusion = Expr::try_from(conclusion).ok()?;
    let result = vst_rewriter_split_smaller_or_equal_to(assert.clone(), v_conclusion)?;
    let result = ctx.fmt(assert_forall_expr.clone(),result.to_string())?;

    acc.add(
        AssistId("split_smaller_or_equal_to", AssistKind::RefactorRewrite),
        "Split smaller or equal to into separate assertions",
        assert_forall_expr.syntax().text_range(),
        |edit| {
            edit.replace(assert_forall_expr.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_split_smaller_or_equal_to(
    assert_forall: AssertForallExpr, 
    conclusion: Expr,
) -> Option<BlockExpr> {

    if !assert_forall.implies_token {
      return None;
    }
    
    let block_expr = 
    {
        let c = *assert_forall.closure_expr.clone();  
        if !c.forall_token {
          panic!("framework internal error: not a forall");
        }


        let (lhs, rhs) = match *c.body {
          Expr::BinExpr(b) => {
              if b.op != BinaryOp::CmpOp(CmpOp::Ord { ordering: Ordering::Less, strict: false }) {
                  dbg!("not an <=");
                  return None;
              }
              (*b.lhs, *b.rhs)
          }
          _ => {dbg!("not a binexpr"); return None;},
        };

        let mut strictly_smaller_closure = *assert_forall.closure_expr.clone();  
        let mut equal_closure = *assert_forall.closure_expr.clone();
        let strictly_smaller_expr:Expr = BinExpr::new(lhs.clone(), BinaryOp::CmpOp(CmpOp::Ord { ordering: Ordering::Less, strict: true }), rhs.clone()).into();
        let equal_expr:Expr = BinExpr::new(lhs, BinaryOp::CmpOp(CmpOp::Eq { negated: false }), rhs).into();

        strictly_smaller_closure.body = Box::new(strictly_smaller_expr); 
        equal_closure.body = Box::new(equal_expr); 

        let mut strictly_smaller_assert_forall = AssertForallExpr::new(strictly_smaller_closure, *assert_forall.block_expr.clone());
        let mut equal_assert_forall = AssertForallExpr::new(equal_closure, *assert_forall.block_expr.clone());
        strictly_smaller_assert_forall.implies_token = true;
        equal_assert_forall.implies_token = true;

        strictly_smaller_assert_forall.expr = Some(Box::new(conclusion.clone()));
        equal_assert_forall.expr = Some(Box::new(conclusion.clone()));

        let mut stmt = StmtList::new();
        stmt.statements.push(strictly_smaller_assert_forall.into());
        stmt.statements.push(equal_assert_forall.into());
        stmt.statements.push(assert_forall.clone().into());
        let blk_expr: BlockExpr = BlockExpr::new(stmt);
        blk_expr
    };
    Some(block_expr) 
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_split_smaller_or_equal_to1() {
        check_assist(
          split_smaller_or_equal_to,
"
spec fn smaller_than_10(x: int) -> bool
{
  x < 10
}

proof fn test_split_smaller_or_equal_to() 
{
  assert forall |x:int| x <$0= 10 implies smaller_than_10(x) by {}
}
",
"
spec fn smaller_than_10(x: int) -> bool
{
  x < 10
}

proof fn test_split_smaller_or_equal_to() 
{
  {
        assert forall|x: int| x < 10 implies smaller_than_10(x) by {};
        assert forall|x: int| x == 10 implies smaller_than_10(x) by {};
        assert forall|x: int| x <= 10 implies smaller_than_10(x) by {};
    }
}
",

        )
    }
}

