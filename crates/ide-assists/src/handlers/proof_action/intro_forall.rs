use crate::{
    assist_context::{AssistContext, Assists},
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, vst::*, AstNode},
    T,
};

/// Change `assert(forall || P )` into 
/// `assert forall || P by {}`
pub(crate) fn intro_forall(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
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
    let result = vst_rewriter_intro_forall(assert.clone())?; // TODO: verusfmt
    let result = ctx.fmt(assert_expr.clone(),result.to_string())?;

    acc.add(
        AssistId("intro_forall", AssistKind::RefactorRewrite),
        "Introduce Assert Forall",
        forall_range,
        |edit| {
            edit.replace(assert_expr.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_intro_forall(assert: AssertExpr) -> Option<AssertForallExpr> {
    // if assertion's expression's top level is not implication, return None
    let assert_forall_expr = match *assert.expr {
        Expr::ClosureExpr(c) => {
          if !c.forall_token {
            dbg!("not a forall");
            return None;
          }
          AssertForallExpr::new(*c.clone(), *assert.block_expr.unwrap_or(Box::new(BlockExpr::new(StmtList::new()))))
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
    fn test_intro_forall_1() {
        check_assist(
          intro_forall,
"
#[verifier::opaque]
spec fn twice(x: int) -> int
{
  x * 2
} 

proof fn test_intro_forall() {
  assert(for$0all|x: int, y: int| twice(x) + twice(y) == x*2 + y*2) by {
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

proof fn test_intro_forall() {
  assert forall|x: int, y: int| twice(x) + twice(y) == x * 2 + y * 2 by {
        reveal(twice);
    }
}
",

        )
    }

    #[test]
    fn test_intro_forall_2() {
        check_assist(
          intro_forall,
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
  assert forall|i: int, j: int| i == j ==> f1(i, j) && f1(i, j) by {};

}
",

        )
    }
}

