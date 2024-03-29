use crate::{AssistContext, Assists};
use ide_db::assists::{AssistId, AssistKind};
use crate::vst_api::vst_ext::*;
use syntax::{ast::{self, vst::*}, AstNode, T};

/*
"Weakest Precondition Step" a.k.a. "Move up assertion"

This proof action allows users to step thourgh an assertion through statements, utilizing the rules below.

Previous statement =
    | Let-binding(simple expression -- i.e., does not have any function call with ensures clause) 
    | Let-binding(expression with spec -- i.e., expression contains a function call with ensures clause) 
    | If-else 
    | Match-statement 
    | Assert 
    | Lemma-Call(i.e., function without return value) 


For each statement, we use the following rules.

Let-binding with simple expression:  
{let x = e; assert(Y);}
rewrites to
{assert(Y[e/x]); let x = e; assert(Y);} 


Let-binding with Function-call:   
create a local scope and let-binds a free variable
which stands for the return value of the function call
inside forall, make implication "ensures ==> original pred".
Note that we intentionally skip "inlines requires clause" as we have a separate proof action for this


If-else and match-statement:
Copy the assertion into each branch/match-arms.


Assert: simple “==>”
{assert(PREV); assert(P);}
rewrites to 
{assert(PREV ==> P); assert(PREV); assert(P);}    ;(TODO: consider adding "Assume" in statement -- use the same rewrite rule here)


Lemma-call 
inline ensures clause and make implication

*/

pub(crate) fn wp_move_assertion(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on assert keyword
    let _ = ctx.at_this_token(T![assert])?;

    // at high level, we 
    // 1) locate the VST node we want to modify(flexible granularity), 2) apply the rewriter for it, and 3) replace the original node with the new node
    // in this case, let's modify statement_list that contains the assertion
    let stmt_list = ctx.find_node_at_offset::<ast::StmtList>()?;
    let v_stmt_list = StmtList::try_from(stmt_list.clone()).ok()?;
    let result = vst_rewriter_wp_move_assertion(ctx, v_stmt_list.clone())?;
    let result = ctx.fmt(stmt_list.clone(),result.to_string())?;

    acc.add(
        AssistId("move_up_assertion", AssistKind::RefactorRewrite),
        "Move up assertion through statements ",
        stmt_list.syntax().text_range(),
        |edit| {
            edit.replace(stmt_list.syntax().text_range(), result);
        },
    )
}

// use only VST in the rewriter
pub(crate) fn vst_rewriter_wp_move_assertion(
    ctx: &AssistContext<'_>,
    stmt_list: StmtList,
) -> Option<StmtList> { // TODO: return VST Node instead of string, to make it easier to use in other places
    // find the assertion of interest
    let assertion = ctx.vst_find_node_at_offset::<AssertExpr, ast::AssertExpr>()?;
    // find the index of the assertion in the statement list
    let index = stmt_list.statements.iter().position(|s| match s {
        Stmt::ExprStmt(e) => match e.expr.as_ref() {
            Expr::AssertExpr(a) => **a == assertion,
            _ => false,
        },
        _ => false,
    })?;

    // Not applicable when the assertion is already at the top
    if index == 0 {
        return None;
    }

    // now match on previous statement to decide how to modify the assertion
    let prev: &Stmt = stmt_list.statements.get(index - 1)?;
    // when is_insert, insert the assertion before the previous statement
    // when !is_insert, replace the previous statement with `new_stmt`
    let (new_stmt, is_insert): (Stmt, bool) = match prev {
        Stmt::LetStmt(l) => {
            let pat = &**l.pat.as_ref()?;
            let init_expr = l.initializer.as_ref();
            let new_stmt: Option<Stmt> = match init_expr {
                // when `init` is a function call with ensures clause, inline ensures clause
                Expr::CallExpr(call) => {
                    let func = ctx.vst_find_fn(call)?;
                    let func_ret_type = func.ret_type?.clone();
                    let first_formal_param = func.param_list.as_ref()?.params.first().clone()?;
                    let first_formal_arg = first_formal_param.pat.as_ref()?;
                    let first_arg = call.arg_list.args.first()?;
                    // TODO: make sure spec function call is directly replaced -- i.e. make sure spec function's ensures is Some; see `?` below
                    if func.ensures_clause.as_ref()?.ensures_token {
                        let mut new_assertion = assertion.clone();
                        let ret_var = func_ret_type.pat?;
                        let ensures: Result<Vec<Expr>, String> = func.ensures_clause?.exprs.clone().iter().map(|e1|
                            vst_map_expr_visitor(e1.clone(), &mut |e2| {
                                if e2.to_string().trim() == ret_var.to_string().trim() {
                                    // assume one arg for now
                                    ctx.vst_expr_from_text(pat.to_string().as_str()).map_or(Err(String::new()), |e| Ok(e))
                                } else if e2.to_string().trim() == first_formal_arg.to_string().trim(){
                                    ctx.vst_expr_from_text(first_arg.to_string().as_str()).map_or(Err(String::new()), |e| Ok(e))
                                }
                                else {
                                    Ok(e2.clone())
                                }
                            })
                        ).collect();

                        // reduce ensures clauses into one &&-ed expr
                        let ensures_anded: Expr = ctx.reduce_exprs(ensures.ok()?)?;
                
                        // ensures ==> original predicate
                        let bin_expr: Expr = BinExpr::new(
                            ensures_anded.clone(),
                            BinaryOp::LogicOp(ast::LogicOp::Imply),
                            *assertion.expr.clone(),
                        ).into();
                        new_assertion.expr = Box::new(bin_expr);
                        let new_stmt:Stmt = new_assertion.into();
                        let simple_let: Stmt = ctx.vst_expr_from_text(format!("let {} :{}", pat, func_ret_type.ty).as_ref())?.into();
                        let mut stmt_list = StmtList::new();
                        stmt_list.statements = vec![simple_let, new_stmt];
                        Some(BlockExpr::new(stmt_list).into()) // is_insert = true
                    } else {
                        None
                    }
                }
                _ => None,
            };
            match new_stmt {
                Some(new_stmt) => (new_stmt, true),
                None => {
                    // when `prev` is let-binding, do subsitution (replace `pat` with `init`)
                    let new_assert = vst_map_expr_visitor(assertion.clone(), &mut |e| {
                        // TODO: do proper usage check in semantic level instead of string match             
                        // TODO: careful with variable name shadowing
                        if e.to_string().trim() == pat.to_string().trim() {
                            Ok(init_expr.clone())
                        } else {
                            Ok(e.clone())
                        }
                    }).ok()?;
                    (new_assert.into(), true)
                },
            }
        }
        Stmt::ExprStmt(exp_stmt) => {
            let exp = exp_stmt.expr.as_ref();
            match exp {
                // prev is another assertion. Generate `(prev) ==> assertion`
                Expr::AssertExpr(prev) => {
                    let mut new_assertion = assertion.clone();
                    let e = assertion.expr.clone();
                    let bin_expr: Expr = BinExpr::new(
                        *prev.expr.clone(),
                        BinaryOp::LogicOp(ast::LogicOp::Imply),
                        *e,
                    ).into();
                    new_assertion.expr = Box::new(bin_expr);
                    (new_assertion.into(), true)
                }
                // prev is if-else. For each branch, insert assertion
                // recursively insert for nested if-else
                Expr::IfExpr(_if_expr) => {
                    let cb = |exp: &mut Expr| {
                        match exp {
                            Expr::BlockExpr(bb) => {
                                bb.stmt_list.statements.push(Stmt::from(assertion.clone()));
                            }
                            _ => (),
                        };
                        return Ok::<Expr, String>(exp.clone());
                    };
                    let new_if_expr = vst_map_expr_visitor(exp.clone(), &cb).ok()?;
                    (new_if_expr.into(), false)
                }
                Expr::MatchExpr(match_expr) => {
                    let adding_assert = Stmt::from(assertion.clone());
                    let mut new_match_expr = match_expr.clone();
                    new_match_expr.match_arm_list.arms.iter_mut().for_each(|arm: &mut MatchArm| {
                        let existing_expr = arm.expr.clone();
                        match *existing_expr {
                            // when match arm is a block, insert assertion at the end
                            Expr::BlockExpr(mut bb) => {
                                bb.stmt_list.statements.push(adding_assert.clone());
                                arm.expr = Box::new(Expr::BlockExpr(bb));
                            }
                            // when match arm is a single expression, convert it to a block and insert the assertion before it
                            _ => {
                                // FIXME: let binding 
                                let mut new_blk = BlockExpr::new(StmtList::new());
                                new_blk.stmt_list.statements = vec![adding_assert.clone()];
                                new_blk.stmt_list.tail_expr = Some(existing_expr.clone());
                                arm.expr = Box::new(Expr::from(new_blk));
                            }
                        }
                    });
                    (Stmt::from(*new_match_expr), false)
                }
                // for lemma calls, do  `(inlined ensures clauses) ==> assertion`
                Expr::CallExpr(call_expr) => {
                    if let Expr::PathExpr(pp) = *call_expr.expr.clone() {
                        let func = ctx.vst_find_fn(&call_expr)?;
                        // TODO: exec functions
                        if !func.fn_mode.as_ref()?.proof_token {
                            return None;
                        }
                        let vst_name_ref: NameRef = *pp.path.segment.name_ref;
                        // inline every ensures clause
                        let ensures: Option<Vec<Expr>> = func.ensures_clause?
                            .clone()
                            .exprs
                            .into_iter()
                            .map(|e| ctx.vst_inline_call(vst_name_ref.clone(), e))
                            .collect();
                        // apply `&&` for all ensures clauses
                        let inlined_ensures = ctx.reduce_exprs(ensures?)?;

                        // generate `ensures ==> assertion`
                        let final_assert = AssertExpr::new(BinExpr::new(
                            inlined_ensures.clone(),
                            BinaryOp::LogicOp(ast::LogicOp::Imply),
                            *assertion.expr.clone(),
                        ));
                        (final_assert.into(), true)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Stmt::Item(_) => return None,
    };

    // insert new assertion in the right place and return
    let mut new_stmt_list = stmt_list.clone();
    if is_insert {
        new_stmt_list.statements.insert(index - 1, new_stmt);
    } else {
        // this is replace
        new_stmt_list.statements[index - 1] = new_stmt;
    }
    return Some(new_stmt_list);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::check_assist;

    // TEST: let-binding
    #[test]
    fn wp_let_bind() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let a: u32 = 1;
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    assert(1 > 10 && 1 < 100);
    let a: u32 = 1;
    assert(a > 10 && a < 100);
}

"#,
        );
    }


    // TEST: assert
    #[test]
    fn wp_assertion_step() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let a: u32 = 1;
    assert(true);
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let a: u32 = 1;
    assert(true ==> a > 10 && a < 100);
    assert(true);
    assert(a > 10 && a < 100);
}

"#,
        );
    }


    // TEST: if-else
    #[test]
    fn wp_if_else() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let mut a: u32 = 1;
    if (a > 10) {
        a = 2;
    };
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a: u32 = 1;
    if (a > 10) {
        a = 2;
        assert(a > 10 && a < 100);
    };
    assert(a > 10 && a < 100);
}

"#,
        );
    }


    // TEST: nested if-else
    #[test]
    fn wp_if_else_nested() {
        check_assist(
            wp_move_assertion,
            r#"
fn foo()
{
    let mut a: u32 = 1;
    if (a > 10) {
        a = 2;
    } else if (a > 100) {
        a = 3;
    } else {
        a = 4;
    };
    ass$0ert(a > 10 && a < 100);
}
"#,
            r#"
fn foo()
{
    let mut a: u32 = 1;
    if (a > 10) {
        a = 2;
        assert(a > 10 && a < 100);
    } else if (a > 100) {
        a = 3;
        assert(a > 10 && a < 100);
    } else {
        a = 4;
        assert(a > 10 && a < 100);
    };
    assert(a > 10 && a < 100);
}

"#,
        );
    }

    // TEST: Lemma Call
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
    assert(v1 * v2 == v2 * v1 ==> false);
    commutative(v1, v2);
    assert(false);
}

"#,
        )
    }

    // TEST: Function Call
    #[test]
    fn wp_function_call() {
        check_assist(
            wp_move_assertion,
            r#"
fn octuple(x1: i8) -> (x8: i8)
requires
    -16 <= x1,
    x1  < 16,
ensures                 
    x8 == 8 * x1,
{
    let x2 = x1 + x1;
    let x4 = x2 + x2;
    x4 + x4
}

fn use_octuple() {
    let two = 2;
    let num = octuple(two);
    ass$0ert(num == 32);        
}
"#,

            r#"
fn octuple(x1: i8) -> (x8: i8)
requires
    -16 <= x1,
    x1  < 16,
ensures                 
    x8 == 8 * x1,
{
    let x2 = x1 + x1;
    let x4 = x2 + x2;
    x4 + x4
}

fn use_octuple() {
    let two = 2;
    {
        let num: i8;
        assert(num == 8 * two ==> num == 32);
    };
    let num = octuple(two);
    assert(num == 32);
}

"#,
        )
    }

    // TEST: match
    #[test]
    fn wp_match() {
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
    };
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
            let foo = 1;
            assert(true);
            foo > 100
        },
    };
    assert(true);
}

"#,
        );
    }

    // TEST: fibo-example
    #[test]
    fn wp_ifelse_fibo() {
        check_assist(
            wp_move_assertion,
            r#"
pub open spec fn fibo(n: nat) -> nat
    decreases n
{
    if n == 0 { 0 } else if n == 1 { 1 }
    else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}

proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires i <= j,
    ensures fibo(i) <= fibo(j),
    decreases j - i
{
    if i < 2 && j < 2 {
    } else if i == j {
    } else if i == j - 1 {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
    };
    a$0ssert(fibo(i) <= fibo(j));
}
"#,
            r#"
pub open spec fn fibo(n: nat) -> nat
    decreases n
{
    if n == 0 { 0 } else if n == 1 { 1 }
    else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}

proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires i <= j,
    ensures fibo(i) <= fibo(j),
    decreases j - i
{
    if i < 2 && j < 2 {
        assert(fibo(i) <= fibo(j));
    } else if i == j {
        assert(fibo(i) <= fibo(j));
    } else if i == j - 1 {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        assert(fibo(i) <= fibo(j));
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
        assert(fibo(i) <= fibo(j));
    };
    assert(fibo(i) <= fibo(j));
}

"#,
        );
    }

//     #[test] // not yet implemented
//     fn wp_assign_easy() {
//         check_assist(
//             wp_move_assertion,
//             r#"
// fn foo()
// {
//     let mut a:u32 = 1;
//     a = 8;
//     ass$0ert(a > 10 && a < 100);
// }
// "#,
//             r#"
// fn foo()
// {
//     let mut a:u32 = 1;
//     assert(8 > 10 && 8 < 100);
//     a = 8;
//     assert(a > 10 && a < 100);
// }
// "#,
//         );
//     }

//     #[test] // not yet implemented
//     fn wp_assign_expr() {
//         check_assist(
//             wp_move_assertion,
//             r#"
// fn foo()
// {
//     let mut a:u32 = 1;
//     a = 8 + 9;
//     ass$0ert(a > 10 && a < 100);
// }
// "#,
//             r#"
// fn foo()
// {
//     let mut a:u32 = 1;
//     assert(8 + 9 > 10 && 8 + 9 < 100);
//     a = 8 + 9;
//     assert(a > 10 && a < 100);
// }
// "#,
//         );
//     }
}

// let stmt
// match e {
// Expr::PathExpr(p) => {
//     if p.to_string() == pat.to_string() {
//         Ok(init_expr.clone())
//     } else {
//         Ok(e.clone())
//     }
// }
// _ => Ok(e.clone()),
// }
