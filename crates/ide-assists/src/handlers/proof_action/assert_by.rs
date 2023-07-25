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

pub(crate) fn assert_by(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let expr = ast::AssertExpr::cast(assert_keyword.parent()?)?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }
    let assert: vst::AssertExpr = vst::AssertExpr::try_from(expr.clone()).ok()?;
    let string = vst_rewriter_assert_to_assert_by(assert.clone())?; // TODO: verusfmt

    acc.add(
        AssistId("assert_by", AssistKind::RefactorRewrite),
        "Add proof block for this assert",
        assert_range,
        |edit| {
            edit.replace(expr.syntax().text_range(), string);
        },
    )
}

pub(crate) fn vst_rewriter_assert_to_assert_by(mut assert: vst::AssertExpr) -> Option<String> {
    // if is already has a "by block", return None
    if assert.by_token {
        return None;
    }
    let new_assert_as_stmt: vst::Stmt = assert.clone().into();
    let mut stmt = vst::StmtList::new();
    stmt.statements.push(new_assert_as_stmt);
    let blk_expr: vst::BlockExpr = vst::BlockExpr::new(stmt);
    assert.by_token = true;
    assert.block_expr = Some(Box::new(blk_expr));
    Some(assert.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn assert_by_composite_condition() {
        check_assist(
            assert_by,
            "
proof fn f() { 
    ass$0ert(x == 3); 
}
            ",
            "
proof fn f() { 
    assert(x == 3) by {
        assert(x == 3);
    } 
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
