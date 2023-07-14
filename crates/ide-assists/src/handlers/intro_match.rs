use std::vec;

use crate::{AssistContext, AssistId, AssistKind, Assists};
use hir::Semantics;
use ide_db::{syntax_helpers::vst_ext::vst_walk_expr, RootDatabase};

use syntax::{
    ast::{self, vst},
    AstNode, T,
};

// get vst node and return vst node
// to do that, we need pointer from vst node to cst nodes
fn type_of_expr_adt(sema: &Semantics<'_, RootDatabase>, expr: &vst::Expr) -> Option<vst::Adt> {
    let expr = expr.cst()?;
    let hir_ty: Vec<hir::Type> = sema.type_of_expr(&expr)?.adjusted().autoderef(sema.db).collect::<Vec<_>>();
    let hir_ty = hir_ty.first()?;
    if let Some(t) = hir_ty.as_adt() {
        let ast_ty: ast::Adt = sema.source(t)?.value;
        return ast_ty.try_into().ok();
    }
    None
}

fn get_enum_type(sema: &Semantics<'_, RootDatabase>, expr: &vst::Expr) -> Option<vst::Enum> {
    let typename = type_of_expr_adt(sema, expr)?;
    if let vst::Adt::Enum(e) = typename {
        return Some(*e.clone());
    }
    None
}

// fn goto_definition
 
pub(crate) fn intro_match(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // trigger on `assert` keyword

    // hmm source_file will be only this file and not the whole project
    // let src = &ctx.source_file;
    // for item in ast::HasModuleItem::items(src) {
    //     // dbg!(&item);
    //     let v_item: ast::generated::vst_nodes::Item = item.try_into().unwrap();
    //     dbg!(&v_item);
    //     println!("{}", &v_item);
    // }

    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let assert_expr = ast::AssertExpr::cast(assert_keyword.parent()?)?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let assert: vst::AssertExpr = vst::AssertExpr::try_from(assert_expr.clone()).ok()?;
    let result = code_transformer_intro_match(ctx, assert.clone())?;
    
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


pub(crate) fn code_transformer_intro_match(
    ctx: &AssistContext<'_>,
    assert: vst::AssertExpr,
) -> Option<String> {
    println!("{}", &assert);
    
    let mut v = vec![];
    let cb = &mut |e: vst::Expr| {
        if let Some(_) = get_enum_type(&ctx.sema, &e) {
            v.push(e.clone());
        } 
    };
    let exp_assert = vst::Expr::AssertExpr(Box::new(assert.clone()));
    // walk over the assertion's predicate, to get expressions of `enum` type.
    vst_walk_expr(&exp_assert, cb);
    dbg!(&v);
    println!("match assertion on: {}", &v[0]);
    let enum_expr_inside_assertion = &v[0];
    let en = get_enum_type(&ctx.sema, enum_expr_inside_assertion)?;
    println!("{}", en);

    let mut match_arms = vec![];
    
    for variant in &en.variant_list.variants {
        println!("{}", variant);
        let vst_path: vst::Path = ast::make::path_from_text(&format!("{}::{}(..)", en.name, variant.name)).try_into().ok()?;
        let path_pat = vst::PathPat {
            path: Box::new(vst_path),
            cst: None,
        };
        let vst_pat = vst::Pat::PathPat(Box::new(path_pat));

        let arm = vst::MatchArm{
            attrs: vec![],
            pat: Some(Box::new(vst_pat)),
            guard: None,
            fat_arrow_token: true,
            expr: Box::new(enum_expr_inside_assertion.clone()),
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
    fn intro_failing_ensures_easy() {
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
}
