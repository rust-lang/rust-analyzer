// use ide_db::syntax_helpers::node_ext::is_pattern_cond;
use crate::{
    assist_context::{AssistContext, Assists}, vst_api::run_verus::VerifResult, AssistId, AssistKind
};
use ide_db::syntax_helpers::vst_ext::vst_map_expr_visitor;
use syntax::{ast::{self, vst::{self, *}, AstNode},T,};


// This version does not comment out dead assertions
// instead, it deletes all of them
pub(crate) fn remove_dead_assertions(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on `proof` keyword
    let proof_keyword = ctx.find_token_syntax_at_offset(T![proof])?;
    let cursor_in_range = proof_keyword.text_range().contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?;
    let v_func = vst::Fn::try_from(func.clone()).ok()?;

    // to avoid panic, run verus only one time (verus is fast)
    let initial_verif_result = ctx.try_verus(&v_func)?;
    // only rewrite succeeding proof
    if !initial_verif_result.is_success {
        return None;
    }

    acc.add(
        AssistId("remove_dead_assertion", AssistKind::RefactorRewrite),
        "Remove Redundant Assertions",
        func.syntax().text_range(),
        |edit| {
            // Run `vst_rewriter_remove_dead_assertions` only when the user explicitly wants it,
            // since it runs Verus multiple times.
            // if we run before user permission, it slows down the interaction significantly
            let result = vst_rewriter_remove_dead_assertions(ctx, v_func, initial_verif_result).expect("vst_rewriter_remove_dead_assertions");
            let result = ctx.fmt(func.clone(),result.to_string()).expect("fmt");
            edit.replace(func.syntax().text_range(),result);
        },
    )
}

pub(crate) fn vst_rewriter_remove_dead_assertions(ctx: &AssistContext<'_>, func: vst::Fn, initial_verif_result: VerifResult) -> Option<vst::Fn> {
    let mut redundant_assertions: Vec<vst::Stmt> = vec![];
    for st in &func.body.as_ref()?.stmt_list.statements {
        if let vst::Stmt::ExprStmt(ref e) = st {
            if let vst::Expr::AssertExpr(_) = *e.expr {
                // try if this is redundant
                redundant_assertions.push(st.clone());
                let modified_fn = rewriter_rm_assertions(&func, &redundant_assertions)?;
                let verif_result = ctx.try_verus(&modified_fn)?;
                if !verif_result.is_success {
                    // verification failed without this assertion
                    // remove this assertion from the list
                    redundant_assertions.pop();
                } else {
                    // verif succeeds without this assertion
                    if verif_result.time > initial_verif_result.time * 2 {
                        // verification time takes a lot longer without this assertion
                        redundant_assertions.pop();
                    } 
                }
            }
        }
    }
    rewriter_rm_assertions(&func, &redundant_assertions)
}

fn rewriter_rm_assertions(func: &vst::Fn, redundant_assertions: &Vec<vst::Stmt>) -> Option<vst::Fn> {
    let mut func = func.clone();

    let cb = |exp: &mut Expr| {
        match exp {
            Expr::BlockExpr(bb) => {
                bb.stmt_list.statements = bb.clone().stmt_list.statements.into_iter().filter(|s| redundant_assertions.iter().all(|r| r.to_string().trim() != s.to_string().trim())).collect();
            }
            _ => (),
        };
        return Ok::<Expr, String>(exp.clone());
    };
    let old_body = *func.clone().body?;
    let new_body = vst_map_expr_visitor(old_body, &cb).ok()?;

    match new_body {
        Expr::BlockExpr(be) => {
            func.body = Some(be);
        },
        _ => {
            return None;
        }
    }
    Some(func)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::check_assist;

    #[test]
    fn comment_one() {
        check_assist(
            remove_dead_assertions,
            "
use vstd::prelude::*;
pr$0oof fn foo(x: nat) 
    ensures 
        x >= 0,
{ 
    assert(x >= 0); 
}

fn main() {}
",
            "
use vstd::prelude::*;
proof fn foo(x: nat)
    ensures
        x >= 0,
{
}


fn main() {}
",
        )
    }

    #[test]
    fn assert_comment_success() {
        check_assist(
            remove_dead_assertions,
            r#"
use vstd::prelude::*;
fn main() {}

pr$0oof fn proof_index(a: u16, offset: u16)
    requires    
        offset < 16,
    ensures
        offset < 16,
{
    assert(offset < 16);
    assert(1 == 1);
    assert(15 < 16);
}
"#,
            r#"
use vstd::prelude::*;
fn main() {}

proof fn proof_index(a: u16, offset: u16)
    requires
        offset < 16,
    ensures
        offset < 16,
{
}

"#,
        );
    }


    #[test]
    fn remove_autogen_asserts_fibo() {
        check_assist(
            remove_dead_assertions,
            "
use vstd::prelude::*;
pub open spec fn fibo(n: nat) -> nat
    decreases n
{
    if n == 0 { 0 } else if n == 1 { 1 }
    else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}


pro$0of fn lemma_fibo_is_monotonic(i: nat, j: nat)
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
        assert(fibo(j) == fibo((j-1) as nat) + fibo((j-2) as nat));
        assert(fibo(i) <= fibo(j));
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
        assert(fibo(i) <= fibo(j));
    };
    assert(fibo(i) <= fibo(j));
}

fn main() {}
",

            "
use vstd::prelude::*;
pub open spec fn fibo(n: nat) -> nat
    decreases n
{
    if n == 0 { 0 } else if n == 1 { 1 }
    else { fibo((n - 2) as nat) + fibo((n - 1) as nat) }
}


proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires
        i <= j,
    ensures
        fibo(i) <= fibo(j),
    decreases j - i,
{
    if i < 2 && j < 2 {
    } else if i == j {
    } else if i == j - 1 {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        assert(fibo(j) == fibo((j - 1) as nat) + fibo((j - 2) as nat));
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
    };
}


fn main() {}
",
        )
    }

}