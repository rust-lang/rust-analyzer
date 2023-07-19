use crate::{AssistContext, Assists};
use ide_db::{
    assists::{AssistId, AssistKind},
    syntax_helpers::vst_ext::*,
};

use syntax::{
    ast::{self, vst},
    AstNode, T,
};

// fn reoslve_local(e: &Expr, ctx: &AssistContext<'_>) -> Option<hir::Local> {
//     let pexpr = ast::PathExpr::cast(e.syntax().clone())?;
//     let pres = ctx.sema.resolve_path(&pexpr.path()?)?;
//     if let PathResolution::Local(local) = pres {
//         Some(local)
//     } else {
//         None
//     }
// }

// fn local_usages(
//     expr: &ast::Expr,
//     target: &hir::Local,
//     ctx: &AssistContext<'_>,
// ) -> Vec<syntax::SyntaxElement> {
//     let mut vec = vec![];
//     let cb = &mut |e: Expr| {
//         if let Some(current) = reoslve_local(&e, ctx) {
//             if current == *target {
//                 vec.push(ted::Element::syntax_element(e.syntax()));
//             }
//         }
//     };
//     walk_expr(&expr, cb);
//     vec
// }

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
    let prev = stmt_list.statements.get(index - 1)?;
    let new_assert;
    match prev {
        vst::Stmt::LetStmt(l) => {
            dbg!("prev is let stmt");
            let pat = &**l.pat.as_ref()?;
            let init_expr = l.initializer.as_ref();

            // when `prev` is let-binding, do subsitution (replace `pat` with `init`)
            new_assert = vst_map_expr_visitor(
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
            dbg!(format!("new_assert: {}", new_assert));
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
                    new_assert = vst::Expr::AssertExpr(Box::new(newer));
                }
                // prev is if-else. For each branch, insert assertion
                // recursively insert for nested if-else
                vst::Expr::IfExpr(_if_expr) => {
                    let cb = |exp : &mut vst::Expr|  {
                        match exp {
                            vst::Expr::BlockExpr(bb) => {
                                bb.stmt_list.statements.push(vst::Stmt::from(vst::ExprStmt::new(vst::Expr::from(assertion.clone()))));
                            },
                            _ => (),
                        };
                        return Ok::<vst::Expr, String>(exp.clone());
                    };
                    let new_if_expr = vst_map_expr_visitor(exp.clone(), &cb).ok()?;
                    let mut new_stmt_list = stmt_list.clone();
                    new_stmt_list.statements[index - 1] = vst::Stmt::ExprStmt(Box::new(vst::ExprStmt::new(new_if_expr)));
                    return Some(new_stmt_list.to_string());
                }
                // for lemma calls, do  `(inlined ensures clauses) ==> assertion`
                vst::Expr::CallExpr(_call_expr) => {
                    // call_expr.
                    return None;
                }
                // CallExpr{e, args} => {
                //     let function = ctx.to_def(e); // get function node from call expression
                //     if function.fn_mode == FnMode::Proof {
                //         let inlined_ensures = function.ensures.into_iter().map(|e| ctx.inline(e, args));
                //         let ensures_combined = inlined_ensures.fold(|e1, e2| Expr::BinExpr(Op::And, e1, e2));
                //         new_assert = AssertExpr{e: Expr::BinExpr(Op::Imply, ensures_combined, assertion.e), requires: assertion.requires, block: assertion.block};
                //     } else {
                //         // if this is not a lemma call, do nothing
                //         return None;
                //     }
                // }
                _ => return None,
            }
        }
        vst::Stmt::Item(_) => return None,
    };

    // insert new assertion in the right place and return
    let new_assert_stmt =
        vst::ExprStmt { expr: Box::new(new_assert), semicolon_token: true, cst: None };
    let mut new_stmt_list = stmt_list.clone();
    new_stmt_list.statements.insert(index - 1, vst::Stmt::ExprStmt(Box::new(new_assert_stmt)));
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
}
