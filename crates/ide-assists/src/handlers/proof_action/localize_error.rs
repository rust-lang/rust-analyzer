use crate::{vst_api, AssistContext, Assists};
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
assuming #[is_variant]


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

pub(crate) fn vst_rewriter_localize_error(
    ctx: &AssistContext<'_>,
    assertion: AssertExpr,
) -> Option<String> {
    let exp = &assertion.expr;
    match &**exp {
        Expr::BinExpr(be) => {
            match be.op {
                BinaryOp::LogicOp(LogicOp::And) => {
                    let left_assert = AssertExpr::new(*be.lhs.clone());
                    let right_assert = AssertExpr::new(*be.rhs.clone());
                    let mut stmts: StmtList = StmtList::new();
                    stmts.statements.push(left_assert.into());
                    stmts.statements.push(right_assert.into());
                    stmts.statements.push(assertion.into());
                    let blk = BlockExpr::new(stmts);
                    return Some(blk.to_string());
                }
                _ => return None,
            }

        }            
        _ => return None,
    };
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn wp_assign_easy() {
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
}
