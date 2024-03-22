// use ide_db::syntax_helpers::node_ext::is_pattern_cond;
use crate::{
    assist_context::{AssistContext, Assists},
    // utils::invert_boolean_expression,
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, vst, AstNode},
    T,
};

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
    let result = vst_rewriter_remove_dead_assertions(ctx, v_func)?;
    let result = ctx.fmt(func.clone(),result.to_string())?;

    acc.add(
        AssistId("remove_dead_assertion", AssistKind::RefactorRewrite),
        "Remove dead assertions",
        func.syntax().text_range(),
        |edit| {
            edit.replace(func.syntax().text_range(),result);
        },
    )
}

// TODO: refactor verus interaction parts, and send it to the user using closure
// that way, it does not run before user explicitly wants it
pub(crate) fn vst_rewriter_remove_dead_assertions(ctx: &AssistContext<'_>, func: vst::Fn) -> Option<vst::Fn> {
    let initial_verif_result = ctx.try_verus(&func)?;
    // only rewrite succeeding proof
    if !initial_verif_result.is_success {
        return None;
    }

    let mut redundant_assertions: Vec<vst::Stmt> = vec![];
    for st in &func.body.as_ref()?.stmt_list.statements {
        if let vst::Stmt::ExprStmt(ref e) = st {
            if let vst::Expr::AssertExpr(_) = *e.expr {
                // try if this is redundant
                dbg!("lets check of this is redundant", st.to_string());
                redundant_assertions.push(st.clone());
                let modified_fn = rewriter_rm_assertions(&func, &redundant_assertions)?;
                dbg!("trying out on", modified_fn.to_string());
                let verif_result = ctx.try_verus(&modified_fn)?;
                if !verif_result.is_success {
                    dbg!("verif fails without this assertion");
                    // verification failed without this assertion
                    // remove this assertion from the list
                    redundant_assertions.pop();
                } else {
                    dbg!("verif succeeds without this assertion");
                    if verif_result.time > initial_verif_result.time * 2 {
                        dbg!("verification time takes a lot longer without this assertion");
                        redundant_assertions.pop();
                    } 
                }
                dbg!("redundant assertions", redundant_assertions.len());
            }
        }
    }
    rewriter_rm_assertions(&func, &redundant_assertions)
}

fn rewriter_rm_assertions(func: &vst::Fn, redundant_assertions: &Vec<vst::Stmt>) -> Option<vst::Fn> {
    let stmts = func.body.as_ref()?.stmt_list.statements.clone(); 
    let mut func = func.clone();
    let filtered_stmts: Vec<vst::Stmt> = stmts.into_iter().filter(|s| redundant_assertions.iter().all(|r| r != s)).collect();
    func.body.as_mut()?.stmt_list.statements = filtered_stmts;  
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
    fn assert_comment_fail() {
        check_assist(
            remove_dead_assertions,
            r#"
use vstd::prelude::*;
fn main() {}

$0proof fn proof_index(a: u16, offset: u16)
    requires    
        offset < 1000,
    ensures
        offset & offset < 1000,
    {
        assert(offset < 2000);
        assert(offset & offset == offset) by (bit_vector);
        assert(offset & offset == offset) by(bit_vector);
    }
"#,
            r#"
use vstd::prelude::*;
fn main() {}

proof fn proof_index(a: u16, offset: u16)
    requires
        offset < 1000,
    ensures
        offset & offset < 1000,
{
    assert(offset & offset == offset) by (bit_vector);
}

"#,
        );
    }

    // TODO: testcase for assertions inside a assert-by-proof-block
}