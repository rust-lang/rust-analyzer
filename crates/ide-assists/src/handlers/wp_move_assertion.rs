use crate::{AssistContext, Assists, vst_api};
use ide_db::{
    assists::{AssistId, AssistKind},
    syntax_helpers::vst_ext::*,
};

use syntax::{
    ast::{self, vst},
    AstNode, T,
};

/*
"move up assertion"

Previous statement =  
Let-binding | IF-else |  Match-statement | Assert | Assume | Lemma/Function-Call 

Let-binding:  assert(X)  TO  assert( (x == y) ==> X) —- (careful with variable name shadowing)
IF-else , Match-statement  :  simply move assertion into each cases
Assert/assume : simple “=>”
Lemma/Function-call :   inline ensures clause and make implication 
*/

pub(crate) fn wp_move_assertion(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let stmt_list = ctx.find_node_at_offset::<ast::StmtList>()?;
    let v_stmt_list = vst::StmtList::try_from(stmt_list.clone()).ok()?;
    let result = vst_transformer_wp_move_assertion(ctx, v_stmt_list.clone())?;

    acc.add(
        AssistId("move_up_assertion", AssistKind::RefactorRewrite),
        "Move up assertion through statements ",
        stmt_list.syntax().text_range(),
        |edit| {
            edit.replace(stmt_list.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_transformer_wp_move_assertion(
    ctx: &AssistContext<'_>,
    stmt_list: vst::StmtList,
) -> Option<String> {
    let assertion = ctx.vst_find_node_at_offset::<vst::AssertExpr, ast::AssertExpr>()?;
    println!("assertion: {}", assertion);
    let index = stmt_list.statements.iter().position(|s| match s {
        vst::Stmt::ExprStmt(e) => match e.expr.as_ref() {
            vst::Expr::AssertExpr(a) => **a == assertion,
            _ => false,
        },
        _ => false,
    })?;
    if index == 0 {
        // assertion is already at the top
        return None;
    }
    let prev: &vst::Stmt = stmt_list.statements.get(index - 1)?;
    let (new_stmt, is_insert) =
    match prev {
        vst::Stmt::LetStmt(l) => {
            dbg!("prev is let stmt");
            let pat = &**l.pat.as_ref()?;
            let init_expr = l.initializer.as_ref();

            // when `prev` is let-binding, do subsitution (replace `pat` with `init`)
            // TODO: careful with variable name shadowing
            let new_assert = vst_map_expr_visitor(
                vst::Expr::AssertExpr(Box::new(assertion.clone())),
                &mut |e| {
                    dbg!(format!("e: {}, pat: {}", e, pat));
                    // TODO: do proper usage check in semantic level
                    if e.to_string().trim() == pat.to_string().trim() {
                        // TODO: trim() is a hack
                        Ok(init_expr.clone())
                    } else {
                        Ok(e.clone())
                    }
                    // match e {
                    // vst::Expr::PathExpr(p) => {
                    //     if p.to_string() == pat.to_string() {
                    //         Ok(init_expr.clone())
                    //     } else {
                    //         Ok(e.clone())
                    //     }
                    // }
                    // _ => Ok(e.clone()),
                    // }
                },
            )
            .ok()?;
            let new_stmt = vst::Stmt::from(vst::ExprStmt::new(new_assert));
            (new_stmt, true)
        }
        vst::Stmt::ExprStmt(exp_stmt) => {
            let exp = exp_stmt.expr.as_ref();
            match exp {
                // prev is another assertion. Generate `(prev) ==> assertion`
                vst::Expr::AssertExpr(prev) => {
                    let mut newer = assertion.clone();
                    let e = assertion.expr.clone();
                    let bin_expr = vst::BinExpr {
                        attrs: vec![],
                        op: vst::BinaryOp::LogicOp(ast::LogicOp::Imply),
                        lhs: prev.expr.clone(),
                        rhs: e,
                        cst: None,
                    };
                    let new_exp = vst::Expr::BinExpr(Box::new(bin_expr));
                    newer.expr = Box::new(new_exp);
                    (vst::Stmt::from(vst::ExprStmt::new(vst::AssertExpr::new(newer))), true)
                }
                // prev is if-else. For each branch, insert assertion
                // recursively insert for nested if-else
                vst::Expr::IfExpr(_if_expr) => {
                    let cb = |exp : &mut vst::Expr|  {
                        match exp {
                            vst::Expr::BlockExpr(bb) => {
                                bb.stmt_list.statements.push(vst::Stmt::from(vst::ExprStmt::new(assertion.clone())));
                            },
                            _ => (),
                        };
                        return Ok::<vst::Expr, String>(exp.clone());
                    };
                    let new_if_expr = vst_map_expr_visitor(exp.clone(), &cb).ok()?;
                    (vst::Stmt::from(vst::ExprStmt::new(new_if_expr)), false)
                }
                vst::Expr::MatchExpr(match_expr) => {
                    let adding_assert = vst::Stmt::from(vst::ExprStmt::new(assertion.clone()));
                    let mut new_match_expr = match_expr.clone();
                    new_match_expr.match_arm_list.arms.iter_mut().for_each(|arm: &mut vst::MatchArm| {
                        let existing_expr = arm.expr.clone();
                        match *existing_expr {
                            // when match arm is a block, insert assertion at the end
                            vst::Expr::BlockExpr(mut bb) => {
                                bb.stmt_list.statements.push(adding_assert.clone());
                                arm.expr = Box::new(vst::Expr::BlockExpr(bb));
                            }
                            // when match arm is a single expression, convert it to a block and insert the assertion before it
                            _ => {
                                let mut new_blk = vst::BlockExpr::new(vst::StmtList::new());
                                new_blk.stmt_list.statements = vec![adding_assert.clone()];
                                new_blk.stmt_list.tail_expr = Some(existing_expr.clone());
                                arm.expr = Box::new(vst::Expr::BlockExpr(Box::new(new_blk)));
                            }
                        }
                    });
                    (vst::Stmt::from(vst::ExprStmt::new(*new_match_expr)), false)
                }
                // for lemma calls, do  `(inlined ensures clauses) ==> assertion`
                vst::Expr::CallExpr(call_expr) => {
                    if let vst::Expr::PathExpr(pp) = *call_expr.expr.clone() {
                        let func = ctx.vst_find_fn(*call_expr.clone())?;
                        // TODO: exec functions
                        if !func.fn_mode.as_ref().unwrap().proof_token {
                            return None;
                        }
                        let vst_name_ref: vst::NameRef = *pp.path.segment.name_ref;
                        println!("vst_name_ref: {}", vst_name_ref);
                        let ensures: Option<Vec<vst::Expr>> = func.ensures_clause?.clone().exprs.into_iter().map(|e| ctx.vst_inline_call(vst_name_ref.clone(), e)).collect();
                        let ensures = ensures?;
                        let inlined_ensures : vst::Expr = ensures.into_iter().reduce(|acc,e| {
                            vst::Expr::BinExpr(Box::new(vst::BinExpr::new(
                                acc,
                                vst::BinaryOp::LogicOp(ast::LogicOp::And),
                                e,
                            )))
                            
                        })?;
                        let final_assert = vst::AssertExpr::new(vst::BinExpr::new(
                            inlined_ensures.clone(),
                            vst::BinaryOp::LogicOp(ast::LogicOp::Imply),
                            *assertion.expr.clone(),
                        ));
                        (vst::Stmt::from(vst::ExprStmt::new(vst::Expr::from(final_assert))), true)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        vst::Stmt::Item(_) => return None,
    };

    // insert new assertion in the right place and return
    let mut new_stmt_list = stmt_list.clone();
    if is_insert {
        new_stmt_list.statements.insert(index - 1, new_stmt);
    } else {
        // this is replace
        new_stmt_list.statements[index - 1] = new_stmt;
    }
    return Some(new_stmt_list.to_string());
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn wp_assign_easy() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    a = 8;
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    assert(8 > 10 && 8 < 100);
    a = 8;
    assert(a > 10 && a < 100);
}
"#,
        );
    }

    #[test]
    fn wp_assign_expr() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    a = 8 + 9;
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    assert(8 + 9 > 10 && 8 + 9 < 100);
    a = 8 + 9;
    assert(a > 10 && a < 100);
}
"#,
        );
    }

    #[test]
    fn wp_let_easy() {
        check_assist(
            wp_move_assertion,
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
    assert(1 > 10 && 1 < 100);
    let a:u32 = 1;
    assert(a > 10 && a < 100);
}
"#,
        );
    }

    #[test]
    fn wp_assertion_easy() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let a:u32 = 1;
    assert(true);
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    assert(1 > 10 && 1 < 100);
    assert(true ==> a > 10 && a < 100);
    assert(true);
    assert(a > 10 && a < 100);
}
"#,
        );
    }
    #[test]
    fn wp_ifelse_easy() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    if  (a  > 10) {
        a = 2;
    }
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    if  (a  > 10) {
        a = 2;
        assert(a > 10 && a < 100);
    }
    ass$0ert(a > 10 && a < 100);
}
"#,
        );
    }

    #[test]
    fn wp_if_else_rec() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    if  (a  > 10) {
        a = 2;
    } else if (a > 100) {
        a = 3;
    } else {
        a = 4;
    }
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a:u32 = 1;
    if  (a  > 10) {
        a = 2;
        assert(a > 10 && a < 100);
    } else if (a > 100) {
        a = 3;
        assert(a > 10 && a < 100);
    } else {
        a = 4;
        assert(a > 10 && a < 100);
    }
    ass$0ert(a > 10 && a < 100);
}
"#,
        );
    }

    #[test]
    fn wp_lemma_call() {
        check_assist(
            wp_move_assertion,
            r#"
proof fn commutative(a: int, b: int)
    ensures a*b == b*a,
{
    assume(false);
}

fn foo()
{
    let v1 = 100;
    let v2 = 200;
    commutative(v1, v2);
    ass$0ert(false);
}
"#,
            r#"
proof fn commutative(a: int, b: int)
    ensures a*b == b*a,
{
    assume(false);
}

fn foo()
{
    let v1 = 100;
    let v2 = 200;
    assert(v1*v2 == v2*v1 ==> false);
    commutative(v1, v2);
    ass$0ert(false);
}
"#)
    }



    #[test]
    fn wp_match_easy() {
        check_assist(
            wp_move_assertion,
            r#"
enum Movement {
    Up(u32),
    Down(u32),
}

proof fn good_move(m: Movement)
{
    match m {
        Movement::Up(v) => v > a,
        Movement::Down(v) => {
            let foo = 1;
            foo > 100
        },
    }
    ass$0ert(true);
}
"#,
            r#"
enum Movement {
    Up(u32),
    Down(u32),
}

proof fn good_move(m: Movement)
{
    match m {
        Movement::Up(v) => {
            assert(true);
            v > a
        },
        Movement::Down(v) => {
            assert(true);
            v > 100
        },
    }
    assert(true);
}
"#,
        );
    }

}
