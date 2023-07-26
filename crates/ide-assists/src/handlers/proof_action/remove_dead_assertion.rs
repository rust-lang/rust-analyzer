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


pub(crate) fn remove_dead_assertions(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let proof_keyword = ctx.find_token_syntax_at_offset(T![proof])?;
    let cursor_in_range = proof_keyword.text_range().contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }


    let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?;
    let v_func = vst::Fn::try_from(func.clone()).ok()?;
    let string = vst_rewriter_remove_dead_assertions(v_func)?; // TODO: verusfmt

    acc.add(
        AssistId("remove_dead_assertion", AssistKind::RefactorRewrite),
        "Remove dead assertions",
        func.syntax().text_range(),
        |edit| {
            edit.replace(func.syntax().text_range(),string);
        },
    )
}

pub(crate) fn vst_rewriter_remove_dead_assertions(mut func: vst::Fn) -> Option<String> {
    None
    // if is already has a "by block", return None
    // let mut redundant_assertions: Vec<vst::Stmt> = vec![];
    // for st in func.body.as_ref()?.stmt_list.statements {
    //     if let vst::Stmt::ExprStmt(ref e) = st {
    //         if let vst::Expr::AssertExpr(_) = *e.expr {
    //             redundant_assertions.push(st.clone());
    //             let modified_fn = rewriter_rm_assertions(&func, &redundant_assertions)?;
                
                
    //         }
    //     }


    // }
    
    // None
}

// fn rewriter_rm_assertions(func: &vst::Fn, redundant_assertions: &Vec<vst::Stmt>) -> Option<vst::Fn> {
//     let mut func = func.clone();
//     let filtered_stmts: Vec<vst::Stmt> = func.body?.stmt_list.statements.into_iter().filter(|s| redundant_assertions.iter().any(|r| r == s)).collect();
//     Some(func)
// }

// // use ide_db::syntax_helpers::node_ext::is_pattern_cond;
// use crate::{
//     assist_context::{AssistContext, Assists},
//     AssistId, AssistKind,
// };
// use syntax::{
//     ast::{self, AstNode},
//     SyntaxKind,
// };

// /*
// Check all assertions in a function
// From top to the bottom, check all assertions.
// If it is redundant, the assertion is commented out
//  */

// pub(crate) fn remove_dead_assertions(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
//     // FIXME: it currently takes too much time to calculate this thing
//     // Furthermore, since this whole thing is processed when user clicks `fn`, this needs to be fixed
//     return None;
//     //
//     let func = ctx.find_node_at_offset::<ast::Fn>()?;
//     let fn_token = func.fn_token()?;
//     let fn_token_range = fn_token.text_range();
//     let cursor_in_range = fn_token_range.contains_range(ctx.selection_trimmed());
//     if !cursor_in_range {
//         return None;
//     }

//     let mut redundant_assertions = vec![];
//     let body = func.body()?;
//     for stmt in body.stmt_list()?.statements() {
//         if let ast::Stmt::ExprStmt(ref stm) = stmt {
//             if let ast::Expr::AssertExpr(..) = stm.expr()? {
//                 let assert_removed_fn = code_transformer_remove_expr_stmt(
//                     func.clone(),
//                     stmt.clone(),
//                     &redundant_assertions,
//                 )?;
//                 if ctx.run_verus_on_fn(assert_removed_fn.fn_token()?)? {
//                     redundant_assertions.push(stmt.clone());
//                 }
//             }
//         }
//     }

//     let fnrange = func.syntax().text_range();
//     acc.add(
//         AssistId("remove_dead_assertions", AssistKind::RefactorRewrite),
//         "Comment out dead assertions",
//         fnrange,
//         |builder| {
//             for stmt in redundant_assertions {
//                 builder.insert(stmt.syntax().text_range().start(), &format!("/* "));
//                 builder.insert(stmt.syntax().text_range().end(), &format!(" */"));
//             }
//         },
//     )
// }

// // a code action that removes
// // 1) `assert_stmt`, 2) any stmts inside `redundant_stmts`.
// // `func` is a placeholder just to make `func` initialized..
// pub(crate) fn code_transformer_remove_expr_stmt(
//     func: ast::Fn,
//     assert_stmt: ast::Stmt,
//     redundant_stmts: &Vec<ast::Stmt>,
// ) -> Option<ast::Fn> {
//     let mut func = func;
//     let assert_stmt = assert_stmt.clone_for_update();
//     for ancestor in assert_stmt.syntax().ancestors() {
//         match ancestor.kind() {
//             SyntaxKind::FN => {
//                 func = ast::Fn::cast(ancestor)?;
//                 break;
//             }
//             _ => (),
//         }
//     }

//     assert_stmt.remove();

//     // TODO:
//     // statements get removed based on string.
//     // this should consider something like offset.
//     // in case we have multiple same assertions
//     for mutable_stm in func.body()?.statements() {
//         if redundant_stmts
//             .iter()
//             .map(|stm| stm.to_string())
//             .any(|stm| stm == mutable_stm.to_string())
//         {
//             mutable_stm.remove();
//         }
//     }

//     Some(func)
// }





#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::check_assist;

    #[test]
    fn comment_one() {
        check_assist(
            remove_dead_assertions,
            "
verus!{
proof fn f() 
    ensures true,
{ 
    ass$0ert(x == 3); 
}
            ",
            "
verus!{
proof fn f() 
    ensures true,
{ 
    // assert(x == 3); 
}
            ",
        )
    }

    #[test]
    fn assert_comment_success() {
        check_assist(
            remove_dead_assertions,
            r#"
#[allow(unused_imports)]
use builtin_macros::*;
#[allow(unused_imports)]
use builtin::*;

mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{modes::*, seq::*, vec::*};

#[verifier(external)]
fn main() {
}

verus! {
    proof fn$0 proof_index(a: u16, offset: u16)
    requires    
        offset < 16
    ensures
        offset < 16
    {
        assert(offset < 16);
        assert(1 == 1);
        assert(15 < 16);
    }
} // verus!
"#,
            r#"
#[allow(unused_imports)]
use builtin_macros::*;
#[allow(unused_imports)]
use builtin::*;

mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{modes::*, seq::*, vec::*};

#[verifier(external)]
fn main() {
}

verus! {
    proof fn proof_index(a: u16, offset: u16)
    requires    
        offset < 16
    ensures
        offset < 16
    {
        /* assert(offset < 16); */
        /* assert(1 == 1); */
        /* assert(15 < 16); */
    }
} // verus!
"#,
        );
    }

    #[test]
    fn assert_comment_fail() {
        check_assist(
            remove_dead_assertions,
            r#"
#[allow(unused_imports)]
use builtin_macros::*;
#[allow(unused_imports)]
use builtin::*;

mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{modes::*, seq::*, vec::*};

#[verifier(external)]
fn main() {
}

verus! {
    proof f$0n proof_index(a: u16, offset: u16)
    requires    
        offset < 1000
    ensures
        offset & offset < 1000
    {
        assert(offset < 2000);
        assert(offset & offset == offset) by (bit_vector);
        assert(offset & offset == offset) by(bit_vector);
    }
} // verus!
"#,
            r#"
#[allow(unused_imports)]
use builtin_macros::*;
#[allow(unused_imports)]
use builtin::*;

mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{modes::*, seq::*, vec::*};

#[verifier(external)]
fn main() {
}

verus! {
    proof fn proof_index(a: u16, offset: u16)
    requires    
        offset < 1000
    ensures
        offset & offset < 1000
    {
        /* assert(offset < 2000); */
        /* assert(offset & offset == offset) by (bit_vector); */
        assert(offset & offset == offset) by(bit_vector);
    }
} // verus!
"#,
        );
    }

    // TODO: testcase for assertions inside a assert-by-proof-block
}