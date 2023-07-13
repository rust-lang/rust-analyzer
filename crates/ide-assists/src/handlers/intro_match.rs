use std::vec;

use crate::{AssistContext, AssistId, AssistKind, Assists};
use hir::{Adt, HasSource, Semantics};
use ide_db::{syntax_helpers::{node_ext::walk_expr, vst_ext::vst_walk_expr}, RootDatabase};

use syntax::{
    ast::{self, edit_in_place::Indent, Expr, HasName, vst},
    AstNode, T,
};

// referenced `add_missing_match_arms`
fn resolve_enum_def(sema: &Semantics<'_, RootDatabase>, expr: &ast::Expr) -> Option<hir::Enum> {
    sema.type_of_expr(expr)?.adjusted().autoderef(sema.db).find_map(|ty| match ty.as_adt() {
        Some(Adt::Enum(e)) => Some(e),
        _ => None,
    })
}

// get vst node and return vst node
// to do that, we need pointer from vst node to cst nodes
fn type_of_expr_adt(sema: &Semantics<'_, RootDatabase>, expr: &ast::Expr) -> Option<ast::Adt> {
    let hir_ty: Vec<hir::Type> = sema.type_of_expr(expr)?.adjusted().autoderef(sema.db).collect::<Vec<_>>();
    let hir_ty = hir_ty.first()?;
    if let Some(t) = hir_ty.as_adt() {
        let ast_ty: ast::Adt = sema.source(t)?.value;
        return Some(ast_ty);
    }
    None
}

// fn get_enum_type(expr: &vst::Expr, items: &Vec<vst::Item>) -> Option<vst::Enum> {
//     for it in items {
//         match it {
//             vst::Item::Enum(e) => {
//                 if e.name.to_string() == expr {
//                     return Some(e.clone());
//                 }
//             }
//             _ => (),
//         }
//     }
//     None
// }

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
    return None;
/* 
    let assert_keyword = ctx.find_token_syntax_at_offset(T![assert])?;
    let assert_expr = ast::AssertExpr::cast(assert_keyword.parent()?)?;
    let assert_range = assert_keyword.text_range();
    let cursor_in_range = assert_range.contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }


    // walk over the assertion's predicate, to get expressions of `enum` type.
    let assert_goal = assert_expr.expr()?;
    let mut v = vec![];
    let cb = &mut |e: vst::Expr| {
        if type_of_expr_adt(&ctx.sema, &e).is_some() {
            v.push(e.clone());
            ()
        } else {
        }
    };
    let assert_vst = vst::Expr::try_from(assert_goal.clone()).ok()?;
    vst_walk_expr(&assert_vst, cb);

    code_transformer_intro_match(assert_expr.clone());


    // now gather code snippet
    let var_of_enum = &v[0];
    let enum_def = resolve_enum_def(&ctx.sema, var_of_enum)?;
    let enum_variants = enum_def.variants(ctx.sema.db).into_iter().collect::<Vec<_>>();
    let enum_name = enum_def.source(ctx.sema.db)?.value.name()?;

    let mut cases = vec![];
    for variant in enum_variants {
        let variant_name = variant.source(ctx.sema.db)?.value.name()?;
        cases.push(format!("{enum_name}::{variant_name}(..) => assert({assert_goal}),"));
    }

    // handle formatting
    let indent = var_of_enum.indent_level();
    let more_indent = indent + 1;
    let seperator = format!("\n{more_indent}");
    let match_cases = cases.join(&seperator);
    let result = format!("match {var_of_enum} {{\n{more_indent}{match_cases}\n{indent}}}");

    // register code change to `acc`
    acc.add(
        AssistId("intro_match", AssistKind::RefactorRewrite),
        "Add match pattern for failed assert on enum ",
        assert_expr.syntax().text_range(),
        |edit| {
            edit.replace(assert_expr.syntax().text_range(), result);
        },
    )
*/
}


pub(crate) fn code_transformer_intro_match(
    assert: ast::AssertExpr,
) -> Option<String> {
    let mut assert: vst::AssertExpr = vst::AssertExpr::try_from(assert).ok()?;
    dbg!(&assert);
    None
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
