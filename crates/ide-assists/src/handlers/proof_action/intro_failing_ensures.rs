use crate::{AssistContext, AssistId, AssistKind, Assists};
use crate::verus_error::*;
use ide_db::syntax_helpers::node_ext::for_each_tail_expr;

use syntax::{
    ast::{self, edit::AstNodeEdit},
     AstNode, 
};

pub(crate) fn intro_failing_ensures(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // setup basic variables
    let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?;
    let body: ast::BlockExpr = func.body()?;
    let ensures: ast::EnsuresClause = func.ensures_clause()?;

    // trigger on "ensures"
    // check if cursor is on "ensures" keyword
    let ensures_keyword = ensures.ensures_token()?;
    let cursor_in_range = ensures_keyword.text_range().contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    // collect failing post-conditions
    let mut failed_posts = vec![];
    let post_fails = filter_post_failuires(&ctx.verus_errors);
    for post in post_fails{
        let post_cond = ctx.find_node_at_given_range::<ast::Expr>(post.failing_post)?;
        let post_assert = format!("assert({post_cond});");
        failed_posts.push(post_assert);
    }
    if failed_posts.len() == 0 {
        return None;
    }

    // Check if the function returns something (to confirm if we need to introduce let-binding)
    let mut ret_name: String = String::new();
    let mut has_ret: bool = false;
    if let Some(ret) = func.ret_type() {
        let ret_pat = ret.pat()?;
        ret_name = format!("{ret_pat}");
        has_ret = true;
    };

    let exit_range = func.body()?.syntax().text_range().end();
    acc.add(AssistId("intro_failing_ensures", AssistKind::RefactorRewrite), "Insert failing ensures clauses to the end", ensures_keyword.text_range(), |edit| {
        if !has_ret {
            let failed_post_concat = failed_posts.join("\n    ");
            edit.insert(exit_range, failed_post_concat);
        } else {
            // when it returns a value, we need to introduce let-binding for each tailing expression
            // when the return expression is if-else or match-statement, we need to introduce let-binding for each cases
            // TODO: do this for "return" (see `wrap_return_type_in_result`)
            
            // collect tail expressions
            let body = ast::Expr::BlockExpr(body);
            let mut exprs_to_bind = Vec::new();
            let tail_cb = &mut |e: &ast::Expr| exprs_to_bind.push(e.clone());
            for_each_tail_expr(&body, tail_cb);

            // for all tail expressions, let-bind and then insert failing postcondition as assertion
            for ret_expr_arg in exprs_to_bind {
                let indent = ret_expr_arg.indent_level();
                let sep = format!("\n{indent}");
                let failed_post_concat = failed_posts.join(&sep);
                let binded = format!("let {ret_name} = {ret_expr_arg};\n{indent}{failed_post_concat}\n{indent}{ret_expr_arg}");
                edit.replace(ret_expr_arg.syntax().text_range(), binded);
            }
        };
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn intro_failing_ensures_easy() {
        check_assist(
            intro_failing_ensures,
            r#"
proof fn my_proof_fun(x: int, y: int)
    requires
        x < 100,
        y < 100,
    ens$0ures
        x + y < 200,
        x + y < 400,
{
    assert(x + y < 600);
}
"#,
            r#"
proof fn my_proof_fun(x: int, y: int)
    requires
        x < 100,
        y < 100,
    ensures
        x + y < 200,
        x + y < 400,
{
    assert(x + y < 600);

    assert(x + y < 400); 
}
"#,
        );
    }

    #[test]
    fn intro_ensure_ret_arg() {
        check_assist(
            intro_failing_ensures,
            r#"
proof fn my_proof_fun(x: int, y: int) -> (sum: int)
    requires
        x < 100,
        y < 100,
    ens$0ures
        sum < 200,
        sum < 300,
        sum < 400,
{
    x + y
}
"#,
            r#"
proof fn my_proof_fun(x: int, y: int) -> (sum: int)
    requires
        x < 100,
        y < 100,
    ensures
        sum < 200,
        sum < 300,
        sum < 400,
{
    let sum = x + y; 
    assert(sum < 200); 
    assert(sum < 300); 
    assert(sum < 400); 
    sum
}
"#,
        );
    }

    #[test]
    fn intro_ensure_fibo() {
        check_assist(
            intro_failing_ensures,
            r#"
proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires
        i <= j,
    e$0nsures
        fibo(i) <= fibo(j),
    decreases j - i
{
    if i < 2 && j < 2 {
    } else if i == j {
    } else if i == j - 1 {
        reveal_with_fuel(fibo, 2);
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
    }
}
"#,
            r#"
proof fn lemma_fibo_is_monotonic(i: nat, j: nat)
    requires
        i <= j,
    ensures
        fibo(i) <= fibo(j),
    decreases j - i
{
    let _ = if i < 2 && j < 2 {
    } else if i == j {
    } else if i == j - 1 {
        reveal_with_fuel(fibo, 2);
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
    } else {
        lemma_fibo_is_monotonic(i, (j - 1) as nat);
        lemma_fibo_is_monotonic(i, (j - 2) as nat);
    }; 
    assert(fibo(i) <= fibo(j)); 
    ()
}
"#,
        );
    }
}