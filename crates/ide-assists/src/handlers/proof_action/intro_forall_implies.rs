use crate::{
    assist_context::{AssistContext, Assists},
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, vst::*, AstNode, LogicOp},
    T,
};

/// Change `assert(forall || P ==> Q)` into 
/// `assert forall || P implies Q`
pub(crate) fn intro_forall_implies(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // if the function name is not inside an assertExpr, return None
    let assert_expr: ast::AssertExpr = ctx.find_node_at_offset()?;

    // trigger on "forall"
    let forall_keyword = ctx.find_token_syntax_at_offset(T![forall])?;
    let forall_range = forall_keyword.text_range();
    let cursor_in_range = forall_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    // now convert to vst nodes
    let assert = AssertExpr::try_from(assert_expr.clone()).ok()?;
    let result = vst_rewriter_intro_forall_implies(assert.clone())?;
    let result = ctx.fmt(assert_expr.clone(),result.to_string())?;

    acc.add(
        AssistId("intro_forall_implies", AssistKind::RefactorRewrite),
        "Introduce Assert forall implies syntax",
        forall_range,
        |edit| {
            edit.replace(assert_expr.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_intro_forall_implies(assert: AssertExpr) -> Option<AssertForallExpr> {
    // if assertion's expression's top level is not implication, return None
    let assert_forall_expr = match *assert.expr {
        Expr::ClosureExpr(c) => {
          if !c.forall_token {
            dbg!("not a forall");
            return None;
          }
          let mut c_clone = *c.clone();

          let (lhs, rhs) = match *c.body {
            Expr::BinExpr(b) => {
                if b.op != BinaryOp::LogicOp(LogicOp::Imply) {
                    dbg!("not an implication");
                    return None;
                }
                (*b.lhs, *b.rhs)
            }
            _ => {dbg!("not a binexpr"); return None;},
          };

          c_clone.body = Box::new(lhs);
          let mut assert_forall = AssertForallExpr::new(c_clone, *assert.block_expr.unwrap_or(Box::new(BlockExpr::new(StmtList::new()))));
          assert_forall.implies_token = true;
          assert_forall.expr = Some(Box::new(rhs));
          assert_forall
        }
        _ => {dbg!("not a ClosureExpr"); return None;},
    };
    Some(assert_forall_expr) 
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_intro_forall_implies1() {
        check_assist(
          intro_forall_implies,
"
#[verifier::opaque]
spec fn twice(x: int) -> int
{
  x * 2
} 

proof fn test_intro_forall_implies() {
  assert(fo$0rall|x:int, y:int| x <= y ==> twice(x) <= twice(y)) by {
    reveal(twice);
  }
}
",
"
#[verifier::opaque]
spec fn twice(x: int) -> int
{
  x * 2
} 

proof fn test_intro_forall_implies() {
  assert forall|x: int, y: int| x <= y implies twice(x) <= twice(y) by {
        reveal(twice);
    }
}
",

        )
    }


    #[test]
    fn test_intro_forall_implies2() {
        check_assist(
          intro_forall_implies,
"
#[verifier::opaque]
spec fn f1(x: int, y: int) -> bool
{
  x * x * x  ==  y * y * y
} 

proof fn test_intro_forall_implies() {
  assert(for$0all|i: int, j: int| i == j ==> f1(i, j) && f1(i, j));

}
",
"
#[verifier::opaque]
spec fn f1(x: int, y: int) -> bool
{
  x * x * x  ==  y * y * y
} 

proof fn test_intro_forall_implies() {
  assert forall|i: int, j: int| i == j implies f1(i, j) && f1(i, j) by {};

}
",

        )
    }
}



