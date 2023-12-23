// use ide_db::syntax_helpers::node_ext::is_pattern_cond;
use crate::{
    assist_context::{AssistContext, Assists},
    // utils::invert_boolean_expression,
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, vst::*, AstNode, LogicOp},
    T,
};

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

    acc.add(
        AssistId("intro_forall", AssistKind::RefactorRewrite),
        "Introduce Assert forall syntax",
        forall_range,
        |edit| {
            edit.replace(assert_expr.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_intro_forall(assert: AssertExpr) -> Option<String> {
    // if assertion's expression's top level is not implication, return None
    let ifstmt = match *assert.expr {
        Expr::ClosureExpr(c) => {

        }
        // Expr::BinExpr(b) => {
        //     if b.op != BinaryOp::LogicOp(LogicOp::Imply) {
        //         dbg!("not an implication");
        //         return None;
        //     }
        //     let rhs_as_assertion = AssertExpr::new(*b.rhs.clone());
        //     let mut blockexpr = BlockExpr::new(StmtList::new());
        //     blockexpr.stmt_list.statements.push(rhs_as_assertion.into());
        //     IfExpr::new(*b.lhs, blockexpr)
        // }
        _ => {dbg!("not a ClosureExpr"); return None;},
    };
    Some(ifstmt.to_string())    
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
fn test_imply_to_if(b: bool) -> (ret: u32) 
  ensures 
    b ==> ret == 2 && !b ==> ret == 1,
{
  let mut ret: u32 = 1;
  if b {
    ret = ret + 1;
  }  
  ass$0ert(b ==> ret == 2);
  ret
}  
",
"
fn test_imply_to_if(b: bool) -> (ret: u32) 
  ensures 
    b ==> ret == 2 && !b ==> ret == 1,
{
  let mut ret: u32 = 1;
  if b {
    ret = ret + 1;
  }  
  if b {
    assert(ret == 2);
  }
  ret
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
