//! Various helper functions to work with VST Nodes
//! Referenced syntax_helpers::node_ext
use itertools::Itertools;
use parser::T;
use syntax::{
    ast::{vst},
};


// referenced Verus' visitor pattern
// #[derive(PartialEq, Eq, Debug)]
// pub(crate) enum VisitorControlFlow<T> {
//     Recurse,
//     Return,
//     Stop(T),
// }


/// Preorder walk all the expression's child expressions.
pub fn vst_walk_expr(expr: &vst::Expr, cb: &mut dyn FnMut(vst::Expr)) {
    vst_preorder_expr(expr, &mut |ev| {
        cb(ev);
        false
    })
}

pub fn vst_map_expr_visitor<EE, FF>(exp: EE, cb: &FF) -> Result<vst::Expr, String>
where
    EE: Into<vst::Expr>,
    FF: Fn(&mut vst::Expr) -> Result<vst::Expr, String>,
{
    let exp: vst::Expr = exp.into();
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

        // note that vst_map_expr_visitor cannot map ifexpr to another thing
        vst::Expr::IfExpr(mut e) => {
            let new_cond = vst_map_expr_visitor(*e.condition.clone(), cb)?;
            let new_then =
                vst_map_expr_visitor(vst::Expr::BlockExpr(Box::new(*e.then_branch.clone())), cb)?;
            let new_else = match e.else_branch.clone() {
                Some(el) => match *el {
                    vst::ElseBranch::Block(blk) => Some(Box::new(vst_map_expr_visitor(
                        vst::Expr::BlockExpr(Box::new(*blk)),
                        cb,
                    )?)),
                    vst::ElseBranch::IfExpr(ife) => {
                        Some(Box::new(vst_map_expr_visitor(vst::Expr::IfExpr(Box::new(*ife)), cb)?))
                    }
                },
                None => None,
            };
            e.condition = Box::new(new_cond);
            if let vst::Expr::BlockExpr(bb) = new_then {
                e.then_branch = bb;
            } else {
                return Err("if then branch is not a block".to_string());
            }

            match new_else {
                Some(vv) => {
                    if let vst::Expr::BlockExpr(bb) = *vv {
                        e.else_branch = Some(Box::new(vst::ElseBranch::Block(bb)));
                    } else if let vst::Expr::IfExpr(ife) = *vv {
                        e.else_branch = Some(Box::new(vst::ElseBranch::IfExpr(ife)));
                    } else {
                        return Err("if else branch is not a block".to_string());
                    }
                }
                None => (),
            }
            vst::Expr::IfExpr(e)
        }
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

// For now, just gather each returning expression 
// TODO: `match` 
// TODO: `return`
pub fn vst_map_each_tail_expr<EE, FF>(exp: EE, cb: &FF) -> Result<vst::Expr, String>
where
    EE: Into<vst::Expr>,
    FF: Fn(&mut vst::Expr) -> Result<vst::Expr, String>,
{
    let exp: vst::Expr = exp.into();
    let res = match exp {
        vst::Expr::IfExpr(mut e) => {
            // does not touch the condition
            let new_then =
            vst_map_each_tail_expr(vst::Expr::BlockExpr(Box::new(*e.then_branch.clone())), cb)?;
            let new_else = match e.else_branch.clone() {
                Some(el) => match *el {
                    vst::ElseBranch::Block(blk) => Some(Box::new(vst_map_each_tail_expr(
                        vst::Expr::BlockExpr(Box::new(*blk)),
                        cb,
                    )?)),
                    vst::ElseBranch::IfExpr(ife) => {
                        Some(Box::new(vst_map_each_tail_expr(vst::Expr::IfExpr(Box::new(*ife)), cb)?))
                    }
                },
                None => None,
            };
            if let vst::Expr::BlockExpr(bb) = new_then {
                e.then_branch = bb;
            } else {
                return Err("if then branch is not a block".to_string());
            }

            match new_else {
                Some(vv) => {
                    if let vst::Expr::BlockExpr(bb) = *vv {
                        e.else_branch = Some(Box::new(vst::ElseBranch::Block(bb)));
                    } else if let vst::Expr::IfExpr(ife) = *vv {
                        e.else_branch = Some(Box::new(vst::ElseBranch::IfExpr(ife)));
                    } else {
                        return Err("if else branch is not a block".to_string());
                    }
                }
                None => (),
            }
            vst::Expr::IfExpr(e)
        }
        _ => {
            dbg!("warning: map each tail expr incomplete");
            dbg!(&exp.to_string());
            cb(&mut exp.clone())?
        }
    };
    Ok(res)
}



/// referenced from syntax_helpers::node_ext::for_each_tail_expr
/// Calls `cb` on each expression inside `expr` that is at "tail position".
/// Does not walk into `break` or `return` expressions.
pub fn for_each_tail_expr(expr: &vst::Expr, cb: &mut dyn FnMut(&vst::Expr)) {
    match expr {
        vst::Expr::IfExpr(e) => {
            match &e.else_branch {
                Some(else_br) => {
                    match &**else_br {
                        vst::ElseBranch::Block(blk) => {
                            for_each_tail_expr(&vst::Expr::BlockExpr(blk.clone()), cb);
                        }
                        vst::ElseBranch::IfExpr(ife) => {
                            for_each_tail_expr(&vst::Expr::IfExpr(ife.clone()), cb);
                        }
                    }
                }
                None => (),
            }
            for_each_tail_expr(&vst::Expr::BlockExpr(e.then_branch.clone()), cb);
        },
        vst::Expr::BlockExpr(e) => {
            if let Some(tail) = &e.stmt_list.tail_expr {
                for_each_tail_expr(&tail, cb)
            }
        }
        // TODO: break, return, 
        _ => cb(expr),

    }


}