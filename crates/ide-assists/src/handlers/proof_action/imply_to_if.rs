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

pub(crate) fn imply_to_if(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let expr = ast::AssertExpr::cast(assert_keyword.parent()?)?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }
    let assert = AssertExpr::try_from(expr.clone());
    dbg!(&assert);
    let assert = assert.ok()?;
    let string = vst_rewriter_imply_to_if(assert.clone())?; // TODO: verusfmt

    acc.add(
        AssistId("imply_to_if", AssistKind::RefactorRewrite),
        "Change implication into if and assert",
        assert_range,
        |edit| {
            edit.replace(expr.syntax().text_range(), string);
        },
    )
}

pub(crate) fn vst_rewriter_imply_to_if(assert: AssertExpr) -> Option<String> {
    // if assertion's expression's top level is not implication, return None
    let ifstmt = match *assert.expr {
        Expr::BinExpr(b) => {
            if b.op != BinaryOp::LogicOp(LogicOp::Imply) {
                dbg!("not an implication");
                return None;
            }
            let rhs_as_assertion = AssertExpr::new(*b.rhs.clone());
            let mut blockexpr = BlockExpr::new(StmtList::new());
            blockexpr.stmt_list.statements.push(rhs_as_assertion.into());
            IfExpr::new(*b.lhs, blockexpr)
        }
        _ => {dbg!("not a binexpr"); return None;},
    };
    Some(ifstmt.to_string())    
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_imply_to_if_1() {
        check_assist(
            imply_to_if,
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
