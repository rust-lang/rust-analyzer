use crate::{vst_api::*, AssistContext, Assists};
use ide_db::{
    assists::{AssistId, AssistKind},
    syntax_helpers::vst_ext::*,
};

use syntax::{
    ast::{self, vst::*, LogicOp},
    AstNode, T,
};

/*
Localize error by splitting assertion

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
    let result = vst_rewriter_localize_error(ctx, v_assertion.clone())?;

    acc.add(
        AssistId("localize_error", AssistKind::RefactorRewrite),
        "Split assertion to localize error",
        assertion.syntax().text_range(),
        |edit| {
            edit.replace(assertion.syntax().text_range(), result);
        },
    )
}

// TODO: match, if, not, quant(forall)
fn split_expr(exp: &Expr) -> Option<Vec<Expr>> {
    match exp {
        Expr::BinExpr(be) => {
            match be.op {
                BinaryOp::LogicOp(LogicOp::And) => {
                    // REVIEW: instead of A and B, consider split into A and A => B
                    // Logically this makes more sense, but it might be confusing to the user
                    Some(vec![*be.lhs.clone(), *be.rhs.clone()])
                }
                BinaryOp::LogicOp(LogicOp::Imply) => {
                    let split_exprs: Vec<Expr> = split_expr(be.lhs.as_ref())?;
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
        Expr::MatchExpr(me) => {
            // is the enum does not have #[is_variant], return None
            // for now, assume #[is_variant]
            // TODO: check for latest syntax
            return None;
        },            
        _ => return None,
    }
}

// TODO: try changing this to use verus
pub(crate) fn vst_rewriter_localize_error(
    ctx: &AssistContext<'_>,
    assertion: AssertExpr,
) -> Option<String> {
    let exp = &assertion.expr;
    let split_exprs = split_expr(exp)?;
    let mut stmts: StmtList = StmtList::new();
    for e in split_exprs {
        let assert_expr = AssertExpr::new(e);
        stmts.statements.push(assert_expr.into());
    }
    stmts.statements.push(assertion.into());
    let blk = BlockExpr::new(stmts);
    return Some(blk.to_string());
}

// maybe register another action with minimization
// TODO: try changing this to use verus
pub(crate) fn vst_rewriter_localize_error_minimized(
    ctx: &AssistContext<'_>,
    assertion: AssertExpr,
) -> Option<String> {
    let this_fn = ctx.vst_find_node_at_offset::<Fn, ast::Fn>()?; 
    let exp = &assertion.expr;
    let split_exprs = split_expr(exp)?;
    let mut stmts: StmtList = StmtList::new();
    for e in split_exprs {
        let assert_expr = AssertExpr::new(e);
        let modified_fn = ctx.replace_statement(&this_fn, assertion.clone(), assert_expr.clone())?;
        if !ctx.try_verus(&modified_fn)? {
            stmts.statements.push(assert_expr.into());
        }
    }
    stmts.statements.push(assertion.into());
    let blk = BlockExpr::new(stmts);
    return Some(blk.to_string());
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn localize_simple_conjunct() {
        check_assist(
            localize_error,
            r#"
fn foo()
{
    let a:u32 = 1;
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let a:u32 = 1;
    assert(a > 10);
    assert(a < 100);
    assert(a > 10 && a < 100);
}
"#,
        );
    }
    #[test]
    fn localize_simple_match() {
        check_assist(
            localize_error,
// before
            r#"
#[derive(PartialEq, Eq)] 
#[is_variant]
pub enum Message {
    Quit(bool),
    Move { x: i32, y: i32 },
    Write(bool),
}

proof fn test_expansion_multiple_call() {
    let x = Message::Move{x: 5, y:6};
    as$0sert(match x {
        Message::Quit(b) => b,
        Message::Move{x, y} => false,
        Message::Write(b) => b,
    });
}            
"#,
// after
            r#"
#[derive(PartialEq, Eq)] 
#[is_variant]
pub enum Message {
    Quit(bool),
    Move { x: i32, y: i32 },
    Write(bool),
}

proof fn test_expansion_multiple_call() {
    let x = Message::Move{x: 5, y:6};
    assert(x.is_Quit() ==> x.get_Quit_0());
    assert(x.is_Move() ==> x.get_Move_x() > x.get_Move_y());
    assert(x.is_Write() ==> x.get_Write_0());
    assert(match x {
        Message::Quit(b) => b,
        Message::Move{x, y} => x > y,
        Message::Write(b) => b,
    });
}
"#,
        );
    }
}
