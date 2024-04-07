//! Various helper functions to work with TOST Nodes (VST Nodes)
//! 
//! Visitor patterns available to map an expression recursively
//! 
//! Referenced syntax_helpers::node_ext
use syntax::ast::vst;


/// Preorder walk all the expression's child expressions.
pub fn vst_walk_expr(expr: &vst::Expr, cb: &mut dyn FnMut(vst::Expr)) {
    vst_preorder_expr(expr, &mut |ev| {
        cb(ev);
        false
    })
}

/// Map a VST Node recursively
pub fn vst_map_expr_visitor<EE, FF>(exp: EE, cb: &mut FF) -> Result<vst::Expr, String>
where
    EE: Into<vst::Expr>,
    FF: FnMut(&mut vst::Expr) -> Result<vst::Expr, String>,
{
    let exp: vst::Expr = exp.into();
    let res = match exp {
        vst::Expr::Literal(_) | vst::Expr::PathExpr(_) => {
            cb(&mut exp.clone())?
        }
        vst::Expr::BinExpr(mut e) => {
            let new_lhs = vst_map_expr_visitor(*e.lhs.clone(), cb)?;
            let new_rhs = vst_map_expr_visitor(*e.rhs.clone(), cb)?;
            e.lhs = Box::new(new_lhs);
            e.rhs = Box::new(new_rhs);
            vst::Expr::BinExpr(e)
        }

        // note that vst_map_expr_visitor cannot map ifexpr to another Expr variant
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
        vst::Expr::AssertExpr(_) => {
            let assert_exp = cb(&mut exp.clone())?;
            match assert_exp {
                vst::Expr::AssertExpr(mut e) => {
                    let new_exp = vst_map_expr_visitor(*e.expr.clone(), cb)?;
                    e.expr = Box::new(new_exp);
                    vst::Expr::AssertExpr(e)
                }
                _ => panic!(),
            }
        }
        vst::Expr::BlockExpr(_) => {
            let newbe = cb(&mut exp.clone())?;
            match newbe {
                vst::Expr::BlockExpr(mut e) => {
                    let new_stmts: Result<Vec<vst::Stmt>,String> = e.clone().stmt_list.statements.into_iter().map(|s| {
                        match s {
                            vst::Stmt::ExprStmt(exprstmt) => {
                                let new_exp = vst_map_expr_visitor(*exprstmt.expr.clone(), cb);
                                match new_exp {
                                    Ok(ne) => Ok(vst::Stmt::ExprStmt(Box::new(vst::ExprStmt::new(ne)))),
                                    Err(ee) => Err(ee),
                                }
                            },
                            _ => Ok(s),
                        }
                    }).collect();
                    e.stmt_list.statements = new_stmts?;
                    vst::Expr::BlockExpr(e)
                }
                _ => panic!(),
            }
        },
        _ => {
            dbg!("note(warning): base case map expr");
            dbg!(&exp.to_string());
            exp.clone()
        }
    };
    Ok(res)
}

/// Preorder walk all the expression's child expressions
pub fn vst_preorder_expr(exp: &vst::Expr, cb: &mut dyn FnMut(vst::Expr) -> bool) {
    cb(exp.clone());
    match exp {
        vst::Expr::ArrayExpr(e) => {
            for expr in &e.exprs {
                vst_preorder_expr(&expr, cb);
            }
            vst_preorder_expr(&e.expr, cb);
        }
        vst::Expr::BinExpr(e) => {
            vst_preorder_expr(&e.lhs, cb);
            vst_preorder_expr(&e.rhs, cb);
        }
        vst::Expr::CallExpr(e) => {
            for arg in &e.arg_list.args {
                vst_preorder_expr(&arg, cb);
            }
        }
        vst::Expr::AssertExpr(e) => {
            vst_preorder_expr(&e.expr, cb);
        }
        _ => {
            dbg!("note(warning): base case vst_preorder_expr");
            dbg!(&exp.to_string());
        }
    }
}

/// Map each tail expressions
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
            dbg!("note(warning): base case vst_map_each_tail_expr");
            dbg!(&exp.to_string());
            cb(&mut exp.clone())?
        }
    };
    Ok(res)
}



/// referenced from syntax_helpers::node_ext::for_each_tail_expr
/// Calls `cb` on each expression inside `expr` that is at "tail position".
/// Does not walk into `break` or `return` expressions.
#[allow(dead_code)]
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