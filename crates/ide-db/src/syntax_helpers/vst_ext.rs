//! Various helper functions to work with VST Nodes
//! Referenced syntax_helpers::node_ext
use itertools::Itertools;
use parser::T;
use syntax::{
    ast::{self, vst, HasLoopBody, MacroCall, PathSegmentKind, VisibilityKind},
    AstNode, AstToken, Preorder, RustLanguage, WalkEvent,
};

/// Preorder walk all the expression's child expressions.
pub fn vst_walk_expr(expr: &vst::Expr, cb: &mut dyn FnMut(vst::Expr)) {
    vst_preorder_expr(expr, &mut |ev| {
        cb(ev);
        false
    })
}

pub fn vst_map_expr_visitor<FF>(exp: vst::Expr, cb: &FF) -> Result<vst::Expr, String>
where
    FF: Fn(&mut vst::Expr) -> Result<vst::Expr, String>,
{
    let res = match exp {
        // vst::Expr::ArrayExpr(e) => {
        //     for expr in &e.exprs {
        //         vst_preorder_expr(&expr, cb);
        //     }
        //     vst_preorder_expr(&e.expr, cb);
        // }
        // vst::Expr::AwaitExpr(e) => todo!(),
        vst::Expr::BinExpr(mut e) => {
            let new_lhs = vst_map_expr_visitor(*e.lhs.clone(), cb)?;
            let new_rhs = vst_map_expr_visitor(*e.rhs.clone(), cb)?;
            e.lhs = Box::new(new_lhs);
            e.rhs = Box::new(new_rhs);
            vst::Expr::BinExpr(e)
        }
        // vst::Expr::BlockExpr(e) => todo!(),
        // vst::Expr::BoxExpr(e) => todo!(),
        // vst::Expr::BreakExpr(e) => todo!(),
        // vst::Expr::CallExpr(e) => {
        //     vst_preorder_expr(&e.expr, cb);
        //     for arg in &e.arg_list.args {
        //         vst_preorder_expr(&arg, cb);
        //     }
        // },
        // vst::Expr::CastExpr(_) => todo!(),
        // vst::Expr::ClosureExpr(_) => todo!(),
        // vst::Expr::ContinueExpr(_) => todo!(),
        // vst::Expr::FieldExpr(_) => todo!(),
        // vst::Expr::ForExpr(_) => todo!(),
        // vst::Expr::IfExpr(_) => todo!(),
        // vst::Expr::IndexExpr(_) => todo!(),
        // vst::Expr::Literal(_) => todo!(),
        // vst::Expr::LoopExpr(_) => todo!(),
        // vst::Expr::MacroExpr(_) => todo!(),
        // vst::Expr::MatchExpr(_) => todo!(),
        // vst::Expr::MethodCallExpr(_) => todo!(),
        // vst::Expr::ParenExpr(_) => todo!(),
        // vst::Expr::PathExpr(e) => {}

        // vst::Expr::PrefixExpr(_) => todo!(),
        // vst::Expr::RangeExpr(_) => todo!(),
        // vst::Expr::RecordExpr(_) => todo!(),
        // vst::Expr::RefExpr(_) => todo!(),
        // vst::Expr::ReturnExpr(_) => todo!(),
        // vst::Expr::TryExpr(_) => todo!(),
        // vst::Expr::TupleExpr(_) => todo!(),
        // vst::Expr::WhileExpr(_) => todo!(),
        // vst::Expr::YieldExpr(_) => todo!(),
        // vst::Expr::YeetExpr(_) => todo!(),
        // vst::Expr::LetExpr(_) => todo!(),
        // vst::Expr::UnderscoreExpr(_) => todo!(),
        // vst::Expr::ViewExpr(_) => todo!(),
        vst::Expr::AssertExpr(mut e) => {
            let new_exp = vst_map_expr_visitor(*e.expr.clone(), cb)?;
            e.expr = Box::new(new_exp);
            vst::Expr::AssertExpr(e)
        }
        // vst::Expr::AssumeExpr(e) => todo!(),
        // vst::Expr::AssertForallExpr(_) => todo!(),
        _ => {
            dbg!("warning: map expr incomplete");
            dbg!(&exp.to_string());
            cb(&mut exp.clone())?
        }
    };
    Ok(res)
}

/// Preorder walk all the expression's child expressions
pub fn vst_preorder_expr(exp: &vst::Expr, cb: &mut dyn FnMut(vst::Expr) -> bool) {
    match exp {
        vst::Expr::ArrayExpr(e) => {
            for expr in &e.exprs {
                vst_preorder_expr(&expr, cb);
            }
            vst_preorder_expr(&e.expr, cb);
        }
        // vst::Expr::AwaitExpr(e) => todo!(),
        vst::Expr::BinExpr(e) => {
            vst_preorder_expr(&e.lhs, cb);
            vst_preorder_expr(&e.rhs, cb);
        }
        // vst::Expr::BlockExpr(e) => todo!(),
        // vst::Expr::BoxExpr(e) => todo!(),
        // vst::Expr::BreakExpr(e) => todo!(),
        vst::Expr::CallExpr(e) => {
            vst_preorder_expr(&e.expr, cb);
            for arg in &e.arg_list.args {
                vst_preorder_expr(&arg, cb);
            }
        }
        // vst::Expr::CastExpr(_) => todo!(),
        // vst::Expr::ClosureExpr(_) => todo!(),
        // vst::Expr::ContinueExpr(_) => todo!(),
        // vst::Expr::FieldExpr(_) => todo!(),
        // vst::Expr::ForExpr(_) => todo!(),
        // vst::Expr::IfExpr(_) => todo!(),
        // vst::Expr::IndexExpr(_) => todo!(),
        // vst::Expr::Literal(_) => todo!(),
        // vst::Expr::LoopExpr(_) => todo!(),
        // vst::Expr::MacroExpr(_) => todo!(),
        // vst::Expr::MatchExpr(_) => todo!(),
        // vst::Expr::MethodCallExpr(_) => todo!(),
        // vst::Expr::ParenExpr(_) => todo!(),
        // vst::Expr::PathExpr(e) => {}

        // vst::Expr::PrefixExpr(_) => todo!(),
        // vst::Expr::RangeExpr(_) => todo!(),
        // vst::Expr::RecordExpr(_) => todo!(),
        // vst::Expr::RefExpr(_) => todo!(),
        // vst::Expr::ReturnExpr(_) => todo!(),
        // vst::Expr::TryExpr(_) => todo!(),
        // vst::Expr::TupleExpr(_) => todo!(),
        // vst::Expr::WhileExpr(_) => todo!(),
        // vst::Expr::YieldExpr(_) => todo!(),
        // vst::Expr::YeetExpr(_) => todo!(),
        // vst::Expr::LetExpr(_) => todo!(),
        // vst::Expr::UnderscoreExpr(_) => todo!(),
        // vst::Expr::ViewExpr(_) => todo!(),
        vst::Expr::AssertExpr(e) => {
            vst_preorder_expr(&e.expr, cb);
        }
        // vst::Expr::AssumeExpr(e) => todo!(),
        // vst::Expr::AssertForallExpr(_) => todo!(),
        _ => {
            dbg!("warning: basecase walk expr incomplete");
            cb(exp.clone());
        }
    }
}
