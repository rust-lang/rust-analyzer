// use ide_db::syntax_helpers::node_ext::is_pattern_cond;
use crate::{
    assist_context::{AssistContext, Assists},
    // utils::invert_boolean_expression,
    AssistId,
    AssistKind,
};
use syntax::{
    ast::{self, AstNode, vst}, T,
};

pub(crate) fn assert_by(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let expr = ast::AssertExpr::cast(assert_keyword.parent()?)?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let string = code_transformer_assert_to_assert_by(expr.clone())?;

    // let formatter = "/home/chanhee/.cargo/bin/rustfmt";
    // let formatted_string = Command::new("echo")
    //     .arg(string.clone())
    //     .arg("|")
    //     .arg(formatter)
    //     .spawn()
    //     .expect("echo command failed to start").stdout.unwrap();
    // dbg!(formatted_string);
    

    acc.add(
        AssistId("assert_by", AssistKind::RefactorRewrite),
        "Add proof block for this assert",
        assert_range,
        |edit| {
            edit.replace(
                expr.syntax().text_range(),
                string,
            );
        },
    )
}

pub(crate) fn code_transformer_assert_to_assert_by(
    assert: ast::AssertExpr,
) -> Option<String> {
    if assert.by_token().is_some() {
        return None;
    }
    let mut assert: vst::AssertExpr = vst::AssertExpr::try_from(assert).ok()?;
    let new_assert = assert.clone();
    let exprstmt: vst::ExprStmt = vst::ExprStmt{
        expr: Box::new(vst::Expr::AssertExpr(Box::new(new_assert))),
        semicolon_token: true,
        cst: None,
    };
    let stmt = vst::StmtList {
        l_curly_token: true,
        attrs: vec![],
        statements: vec![vst::Stmt::ExprStmt(Box::new(exprstmt))],
        tail_expr: None,
        r_curly_token: true,
        cst: None,
    };
    let blk_expr: vst::BlockExpr = vst::BlockExpr {
        stmt_list: Box::new(stmt),
        attrs: vec![],
        label: None,
        try_token: false,
        unsafe_token: false,
        async_token: false,
        const_token: false,
        cst: None,
    };
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