use stdx::format_to;
use hir::HirDisplay;
use syntax::{
    SyntaxKind,
    ast::{self, edit_in_place::Indent, HasArgList, Pat, Expr, BinaryOp, ArithOp},
    AstNode,
};
use ide_db::helpers::node_ext::walk_pat;

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: convert_if_to_filter
//
// Converts a if into a filter when placed in a for_each().
// ```
// # //- minicore: iterators
// # use core::iter;
// fn main() {
//     let it = core::iter::repeat(92);
//     it.for_each$0(|x| {
//         if x > 4 {
//             println!("{}", x);
//         };
//     });
// }
// ```
// ->
// ```
// # use core::iter;
// fn main() {
//     let it = core::iter::repeat(92);
//     it.filter(|&x| x > 4).for_each(|x| {
//         println!("{}", x);
//     });
// }
// ```
pub(crate) fn convert_if_to_filter(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let method = ctx.find_node_at_offset::<ast::MethodCallExpr>()?;

    let closure = match method.arg_list()?.args().next()? {
        ast::Expr::ClosureExpr(expr) => expr,
        _ => return None,
    };

    let (method, receiver) = validate_method_call_expr(ctx, method)?;

    let param_list = closure.param_list()?;
    let param = param_list.params().next()?.pat()?;
    let body = closure.body()?;

    let range = method.syntax().text_range();

    let if_expr = match body.clone() {
        Expr::IfExpr(if_expr) => {
            if_expr
        },
        Expr::BlockExpr(block) => {
            let mut stmts = block.statements();
            let fst_stmt = stmts.next()?;
            continue_iff(stmts.next().is_none())?; // Only one statement
            // First statement is an expression...
            let expr_stmt = match fst_stmt {
                ast::Stmt::ExprStmt(expr_stmt) => expr_stmt,
                _ => return None,
            };

            // ...and even an if clause...
            let expr = expr_stmt.expr()?;
            let if_expr = match expr {
                ast::Expr::IfExpr(my_if_expr) => my_if_expr,
                _ => return None,
            };
            if_expr
        },
        _ => return None,
    };

    let condition = if_expr.condition()?; // ... with a condition...
    continue_iff(if_expr.else_branch().is_none()); // ... and no else branch...
    let then_branch = if_expr.then_branch()?; // ... and a then branch

    acc.add(
        AssistId("convert_if_to_filter", AssistKind::RefactorRewrite),
        "Replace this `if { ... }` with a `filter()`",
        range,
        |builder| {
            let indent = method.indent_level();
            
            let mut buf = String::new();
            // Recursively remove unnecessary `mut`s in the parameter
            let pat_filter = param.clone_for_update();
            let mut to_be_removed = vec![];
            walk_pat(&pat_filter, &mut |cb|
                if let Pat::IdentPat(ident) = cb {
                    if let Some(mut_token) = ident.mut_token() {
                        to_be_removed.push(mut_token);
                    }
                }
            );
            for mut_token in to_be_removed.into_iter() {
                if let Some(ws) = mut_token.next_token().filter(|it| it.kind() == SyntaxKind::WHITESPACE) {
                    ws.detach();
                }
                mut_token.detach();
            }
            format_to!(buf, "{}.filter(|&{}| {})", receiver, pat_filter, condition);

            // Because we removed a if block, reident accordingly the rest of the block
            let block = then_branch.clone_for_update();
            block.reindent_to(indent);

            format_to!(buf, ".for_each(|{}| {})", param, block);

            builder.replace(range, buf)
        },
    )
}

// Assist: convert_sum_call
//
// Converts a sum into a sum().
// ```
// # //- minicore: iterators
// # use core::iter;
// fn main() {
//     let it = core::iter::repeat(92);
//     let mut val: usize = 0;
//     it.for_each$0(|x| val += x);
// }
// ```
// ->
// ```
// # use core::iter;
// fn main() {
//     let it = core::iter::repeat(92);
//     let mut val: usize = 0;
//     val += it.sum::<usize>();
// }
// ```
pub(crate) fn convert_sum_call(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let method = ctx.find_node_at_offset::<ast::MethodCallExpr>()?;

    let closure = match method.arg_list()?.args().next()? {
        ast::Expr::ClosureExpr(expr) => expr,
        _ => return None,
    };

    let (method, receiver) = validate_method_call_expr(ctx, method)?;

    let param_list = closure.param_list()?;
    let param = param_list.params().next()?.pat()?;
    let body = closure.body()?;

    let range = method.syntax().text_range();
    let module = ctx.sema.scope(param.syntax()).module()?;

    let binexpr = match body.clone() {
        Expr::BinExpr(expr) => expr,
        Expr::BlockExpr(block) => {
            let mut stmts = block.statements();
            let fst_stmt = stmts.next()?;
            continue_iff(stmts.next().is_none())?; // Only one statement
            // First statement is an expression...
            let expr_stmt = match fst_stmt {
                ast::Stmt::ExprStmt(expr_stmt) => expr_stmt,
                _ => return None,
            };

            // ...and even a binary expr...
            let expr = expr_stmt.expr()?;
            let my_bin_expr = match expr {
                ast::Expr::BinExpr(my_bin_expr) => my_bin_expr,
                _ => return None,
            };
            my_bin_expr
        },
        _ => return None,
    };
    let op = match binexpr.op_kind()? {
        BinaryOp::Assignment { op } => op?,
        _ => return None,
    };
    match op {
        ArithOp::Add => (),
        _ => return None,
    }
    continue_iff(format!("{}", binexpr.rhs()?) == format!("{}", param))?;
    let sum = binexpr.lhs()?;

    let ty = ctx.sema.type_of_pat(&param)?.adjusted();

    // Fully unresolved or unnameable types can't be annotated
    if (ty.contains_unknown() && ty.type_arguments().count() == 0) || ty.is_closure() {
        return None;
    }

    let inferred_type = ty.display_source_code(ctx.db(), module.into()).ok()?;

    acc.add(
        AssistId("convert_sum_call", AssistKind::RefactorRewrite),
        "Replace this sum in disguise with a `sum()` call",
        range,
        |builder| {
            let mut buf = String::new();
            format_to!(buf, "{} += {}.sum::<{}>()", sum, receiver, inferred_type);
            builder.replace(range, buf)
        },
    )
}

// Assist: convert_all_call
//
// Replace with an all() call when possible.
// ```
// # //- minicore: iterators
// # use core::iter;
// fn main() {
//     let it = core::iter::repeat(92);
//     let mut val: usize = 0;
//     it.for_each$0(|x| val &= x > 0);
// }
// ```
// ->
// ```
// # use core::iter;
// fn main() {
//     let it = core::iter::repeat(92);
//     let mut val: usize = 0;
//     val &= it.all(|&x| x > 0);
// }
// ```
pub(crate) fn convert_all_call(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let method = ctx.find_node_at_offset::<ast::MethodCallExpr>()?;

    let closure = match method.arg_list()?.args().next()? {
        ast::Expr::ClosureExpr(expr) => expr,
        _ => return None,
    };

    let (method, receiver) = validate_method_call_expr(ctx, method)?;

    let param_list = closure.param_list()?;
    let param = param_list.params().next()?.pat()?;
    let body = closure.body()?;

    let range = method.syntax().text_range();

    let binexpr = match body.clone() {
        Expr::BinExpr(expr) => expr,
        Expr::BlockExpr(block) => {
            let mut stmts = block.statements();
            let fst_stmt = stmts.next()?;
            continue_iff(stmts.next().is_none())?; // Only one statement
            // First statement is an expression...
            let expr_stmt = match fst_stmt {
                ast::Stmt::ExprStmt(expr_stmt) => expr_stmt,
                _ => return None,
            };

            // ...and even a binary expr...
            let expr = expr_stmt.expr()?;
            let my_bin_expr = match expr {
                ast::Expr::BinExpr(my_bin_expr) => my_bin_expr,
                _ => return None,
            };
            my_bin_expr
        },
        _ => return None,
    };
    let op = match binexpr.op_kind()? {
        BinaryOp::Assignment { op } => op?,
        _ => return None,
    };
    match op {
        ArithOp::BitAnd => (),
        _ => return None,
    }
    let rhs = binexpr.rhs()?;
    let sum = binexpr.lhs()?;

    acc.add(
        AssistId("convert_all_call", AssistKind::RefactorRewrite),
        "Replace this with a `all()` call",
        range,
        |builder| {
            let mut buf = String::new();
            format_to!(buf, "{} &= {}.all(|&{param}| {})", sum, receiver, rhs);
            builder.replace(range, buf)
        },
    )
}


fn validate_method_call_expr(
    ctx: &AssistContext,
    expr: ast::MethodCallExpr,
) -> Option<(ast::Expr, ast::Expr)> {
    let name_ref = expr.name_ref()?;
    if !name_ref.syntax().text_range().contains_range(ctx.selection_trimmed()) {
        cov_mark::hit!(test_for_each_not_applicable_invalid_cursor_pos);
        return None;
    }
    if name_ref.text() != "for_each" {
        return None;
    }


    let receiver = expr.receiver()?;
    let expr = ast::Expr::MethodCallExpr(expr);
    
    Some((expr, receiver))
}

fn continue_iff(b: bool) -> Option<()> {
    if b { Some(()) } else { None }
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn if_to_filter() {
        check_assist(
            convert_if_to_filter,
            r#"
fn main() {
    let it = core::iter::repeat((92,42));
    it.for_each$0(|(mut i,j)| {
        if (i*j)%3 == 2 {
            i *= 2;
        };
    });
}"#,
            r#"
fn main() {
    let it = core::iter::repeat((92,42));
    it.filter(|&(i,j)| (i*j)%3 == 2).for_each(|(mut i,j)| {
        i *= 2;
    });
}"#,
        )
    }

    #[test]
    fn add_sum_call() {
        check_assist(
            convert_sum_call,
            r#"
fn main() {
    let it = core::iter::repeat(92);
    let mut a: usize = 0;
    it.for_each$0(|x| {
        a += x;
    });
}"#,
            r#"
fn main() {
    let it = core::iter::repeat(92);
    let mut a: usize = 0;
    a += it.sum::<usize>();
}"#,
        )
    }

    #[test]
    fn add_all_call() {
        check_assist(
            convert_all_call,
            r#"
fn main() {
    let it = core::iter::repeat(92);
    let mut a = true;
    it.for_each$0(|x| {
        a &= x > 0;
    });
}"#,
            r#"
fn main() {
    let it = core::iter::repeat(92);
    let mut a = true;
    a &= it.all(|&x| x > 0);
}"#,
        )
    }
}
