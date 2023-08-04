use std::vec;

use crate::{AssistContext, AssistId, AssistKind, Assists};
use ide_db::syntax_helpers::vst_ext::vst_walk_expr;

use syntax::{
    ast::{self, vst},
    AstNode, T,
};

pub(crate) fn intro_match(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on `assert` keyword
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let assert_expr = ast::AssertExpr::cast(assert_keyword.parent()?)?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let assert: vst::AssertExpr = vst::AssertExpr::try_from(assert_expr.clone()).ok()?;
    let result = vst_rewriter_intro_match(ctx, assert.clone())?;

    // register code change to `acc`
    acc.add(
        AssistId("intro_match", AssistKind::RefactorRewrite),
        "Add match pattern for failed assert on enum ",
        assert_expr.syntax().text_range(),
        |edit| {
            edit.replace(assert_expr.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_intro_match(
    ctx: &AssistContext<'_>,
    assert: vst::AssertExpr,
) -> Option<String> {
    dbg!("vst_rewriter_intro_match");
    let mut v = vec![];
    let cb = &mut |e: vst::Expr| {
        if let Some(_) = ctx.type_of_expr_enum(&e) {
            v.push(e.clone());
        }
    };
    let exp_assert = vst::Expr::AssertExpr(Box::new(assert.clone()));
    // walk over the assertion's predicate, to get expressions of `enum` type.
    vst_walk_expr(&exp_assert, cb);
    if v.len() == 0 {
        return None;
    }
    let enum_expr_inside_assertion = &v[0];
    let en = ctx.type_of_expr_enum(enum_expr_inside_assertion)?;
    let mut match_arms = vec![];
    for variant in &en.variant_list.variants {
        let vst_path: vst::Path =
            ctx.vst_path_from_text(&format!("{}::{}(..)", en.name, variant.name))?;
        let path_pat = vst::PathPat { path: Box::new(vst_path), cst: None };
        let vst_pat = vst::Pat::PathPat(Box::new(path_pat));

        let arm = vst::MatchArm {
            attrs: vec![],
            pat: Some(Box::new(vst_pat)),
            guard: None,
            fat_arrow_token: true,
            expr: Box::new(vst::Expr::AssertExpr(Box::new(assert.clone()))),
            comma_token: true,
            cst: None,
        };
        println!("{}", &arm);
        match_arms.push(arm);
    }

    let match_stmt = vst::MatchExpr {
        match_token: true,
        expr: Box::new(enum_expr_inside_assertion.clone()),
        match_arm_list: Box::new(vst::MatchArmList {
            attrs: vec![],
            l_curly_token: true,
            arms: match_arms,
            r_curly_token: true,
            cst: None,
        }),
        attrs: vec![],
        cst: None,
    };

    println!("{}", &match_stmt);

    Some(match_stmt.to_string())
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn intro_match1() {
        check_assist(
            intro_match,
            r#"
enum Movement {
    Up(u32),
    Down(u32),
}

spec fn is_good_move(m: Movement) -> bool {
    match m {
        Movement::Up(v) => v > 100,
        Movement::Down(v) => v > 100,
    }
}

proof fn good_move(m: Movement)
{
    ass$0ert(is_good_move(m));
}
"#,
            r#"
enum Movement {
    Up(u32),
    Down(u32),
}

spec fn is_good_move(m: Movement) -> bool {
    match m {
        Movement::Up(v) => v > 100,
        Movement::Down(v) => v > 100,
    }
}

proof fn good_move(m: Movement)
{
    match m {
        Movement::Up(..) => assert(is_good_move(m)),
        Movement::Down(..) => assert(is_good_move(m)),
    };
}
"#,
        );
    }

    #[test]
    fn intro_match2() {
        check_assist(
            intro_match,
            r#"
enum Movement {
    Up(u32),
    Down(u32),
}

spec fn is_good_move(m: Movement, a: int) -> bool {
    match m {
        Movement::Up(v) => v > a,
        Movement::Down(v) => v > 100,
    }
}

proof fn good_move(m: Movement)
{
    ass$0ert(is_good_move(m, 100));
}
"#,
            r#"
enum Movement {
    Up(u32),
    Down(u32),
}

spec fn is_good_move(m: Movement, a: int) -> bool {
    match m {
        Movement::Up(v) => v > a,
        Movement::Down(v) => v > 100,
    }
}

proof fn good_move(m: Movement)
{
    match m {
        Movement::Up(..) => assert(is_good_move(m, 100)),
        Movement::Down(..) => assert(is_good_move(m, 100)),
    };
}
"#,
        );
    }


// TODO: this test is failing, but it should work
    #[test]
    fn intro_match3() {
        check_assist(
            intro_match,
            r#"
verus!{
    #[derive(PartialEq, Eq)] 
    pub enum Message {
        Quit(bool),
        Move { x: i32, y: i32 },
        Write(bool),
    }
    
    spec fn is_good_integer_3(x: int) -> bool 
    {
        x >= 0 && x != 5
    }
    
    spec fn is_good_message(msg:Message) -> bool {
        match msg {
            Message::Quit(b) => b,
            Message::Move{x, y} => is_good_integer_3( (x as int)  - (y as int)),
            Message::Write(b) => b,
        }
    }
    
    proof fn test_expansion_multiple_call() {
      let x = Message::Move{x: 5, y:6};
      as$0sert(is_good_message(x));
    }
}
"#,

r#"
verus!{
    #[derive(PartialEq, Eq)] 
    pub enum Message {
        Quit(bool),
        Move { x: i32, y: i32 },
        Write(bool),
    }
    
    spec fn is_good_integer_3(x: int) -> bool 
    {
        x >= 0 && x != 5
    }
    
    spec fn is_good_message(msg:Message) -> bool {
        match msg {
            Message::Quit(b) => b,
            Message::Move{x, y} => is_good_integer_3( (x as int)  - (y as int)),
            Message::Write(b) => b,
        }
    }
    
    proof fn test_expansion_multiple_call() {
      let x = Message::Move{x: 5, y:6};
      match x {
        Message::Quit(..) => assert(is_good_message(x)),
        Message::Move{..} => assert(is_good_message(x)),
        Message::Write(..) => assert(is_good_message(x)),
      };
    }
}
"#
        );
    }
}
