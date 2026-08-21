use ide_db::assists::{AssistId, GroupLabel};
use syntax::{
    AstNode, T,
    ast::{self, ArithOp, BinaryOp, HasArgList, UnaryOp, syntax_factory::SyntaxFactory},
};

use crate::{
    assist_context::{AssistContext, Assists},
    utils::wrap_paren,
};

// Assist: replace_arith_with_checked
//
// Replaces arithmetic on integers with the `checked_*` equivalent.
//
// ```
// fn main() {
//   let x = 1 $0+ 2;
// }
// ```
// ->
// ```
// fn main() {
//   let x = 1.checked_add(2);
// }
// ```
pub(crate) fn replace_arith_with_checked(
    acc: &mut Assists,
    ctx: &AssistContext<'_, '_>,
) -> Option<()> {
    replace_arith(acc, ctx, ArithKind::Checked)
}

// Assist: replace_arith_with_saturating
//
// Replaces arithmetic on integers with the `saturating_*` equivalent.
//
// ```
// fn main() {
//   let x = 1 $0+ 2;
// }
// ```
// ->
// ```
// fn main() {
//   let x = 1.saturating_add(2);
// }
// ```
pub(crate) fn replace_arith_with_saturating(
    acc: &mut Assists,
    ctx: &AssistContext<'_, '_>,
) -> Option<()> {
    replace_arith(acc, ctx, ArithKind::Saturating)
}

// Assist: replace_arith_with_strict
//
// Replaces arithmetic on integers with the `strict_*` equivalent.
//
// ```
// fn main() {
//   let x = 1 $0+ 2;
// }
// ```
// ->
// ```
// fn main() {
//   let x = 1.strict_add(2);
// }
// ```
pub(crate) fn replace_arith_with_strict(
    acc: &mut Assists,
    ctx: &AssistContext<'_, '_>,
) -> Option<()> {
    replace_arith(acc, ctx, ArithKind::Strict)
}

// Assist: replace_arith_with_wrapping
//
// Replaces arithmetic on integers with the `wrapping_*` equivalent.
//
// ```
// fn main() {
//   let x = 1 $0+ 2;
// }
// ```
// ->
// ```
// fn main() {
//   let x = 1.wrapping_add(2);
// }
// ```
pub(crate) fn replace_arith_with_wrapping(
    acc: &mut Assists,
    ctx: &AssistContext<'_, '_>,
) -> Option<()> {
    replace_arith(acc, ctx, ArithKind::Wrapping)
}

fn replace_arith(acc: &mut Assists, ctx: &AssistContext<'_, '_>, kind: ArithKind) -> Option<()> {
    let arith = parse_arith_expr(ctx)?;
    let op_expr = arith.root().syntax().clone();

    if !arith.has_int_operands(ctx) {
        return None;
    }

    acc.add_group(
        &GroupLabel("Replace arithmetic...".into()),
        kind.assist_id(),
        kind.label(),
        op_expr.text_range(),
        |builder| {
            let editor = builder.make_editor(&op_expr);
            let make = editor.make();
            let method_name = kind.method_name(&arith);

            let receiver =
                wrap_paren(arith.receiver().clone(), make, ast::prec::ExprPrecedence::Postfix);

            let arg = arith.adjust_arg(ctx, make);
            let mut arith_expr = make
                .expr_method_call(receiver, make.name_ref(&method_name), make.arg_list(arg))
                .into();
            if let Some(lhs) = arith.lhs_for_assign() {
                arith_expr = make.expr_assignment(lhs, arith_expr).into();
            }
            editor.replace(op_expr, arith_expr.syntax());
            builder.add_file_edits(ctx.vfs_file_id(), editor);
        },
    )
}

fn is_primitive_int_or_ref(ctx: &AssistContext<'_, '_>, expr: &ast::Expr) -> bool {
    match ctx.sema.type_of_expr(expr) {
        Some(ty) => ty.original.strip_reference().is_int_or_uint(),
        _ => false,
    }
}

/// An arithmetic expression the assist can rewrite into an overflow-checking
/// method call: a binary operation (`1 + 2`), a `.pow` call (`x.pow(2)`) or a
/// unary negation (`-x`).
enum ArithExpr {
    Binary { expr: ast::Expr, lhs: ast::Expr, op: ArithOp, is_assign: bool, rhs: ast::Expr },
    Pow { expr: ast::Expr, receiver: ast::Expr, exponent: ast::Expr },
    Neg { expr: ast::Expr, operand: ast::Expr },
}

impl ArithExpr {
    /// The whole expression that gets replaced by the method call.
    fn root(&self) -> &ast::Expr {
        match self {
            ArithExpr::Binary { expr, .. }
            | ArithExpr::Pow { expr, .. }
            | ArithExpr::Neg { expr, .. } => expr,
        }
    }

    /// The expression the rewritten method is called on.
    fn receiver(&self) -> &ast::Expr {
        match self {
            ArithExpr::Binary { lhs, .. } => lhs,
            ArithExpr::Pow { receiver, .. } => receiver,
            ArithExpr::Neg { operand, .. } => operand,
        }
    }

    fn has_int_operands(&self, ctx: &AssistContext<'_, '_>) -> bool {
        match self {
            ArithExpr::Binary { lhs, rhs, .. } => {
                is_primitive_int_or_ref(ctx, lhs) && is_primitive_int_or_ref(ctx, rhs)
            }
            ArithExpr::Pow { receiver, exponent, .. } => {
                is_primitive_int_or_ref(ctx, receiver) && is_primitive_int_or_ref(ctx, exponent)
            }
            ArithExpr::Neg { operand, .. } => is_primitive_int_or_ref(ctx, operand),
        }
    }

    /// The argument(s) of the rewritten method call.
    fn adjust_arg(&self, ctx: &AssistContext<'_, '_>, make: &SyntaxFactory) -> Vec<ast::Expr> {
        match self {
            ArithExpr::Binary { rhs, .. } | ArithExpr::Pow { exponent: rhs, .. } => {
                vec![strip_reference(ctx, make, rhs.clone())]
            }
            ArithExpr::Neg { .. } => Vec::new(),
        }
    }

    fn lhs_for_assign(&self) -> Option<ast::Expr> {
        match self {
            ArithExpr::Binary { lhs, is_assign: true, .. } => Some(lhs.clone()),
            _ => None,
        }
    }

    /// The method suffix, e.g. `add`, `pow` or `neg`.
    fn method_suffix(&self) -> &'static str {
        match self {
            ArithExpr::Binary { op, .. } => match op {
                ArithOp::Add => "add",
                ArithOp::Sub => "sub",
                ArithOp::Mul => "mul",
                ArithOp::Div => "div",
                _ => unreachable!("this function should only be called with +, -, / or *"),
            },
            ArithExpr::Pow { .. } => "pow",
            ArithExpr::Neg { .. } => "neg",
        }
    }
}

fn strip_reference(
    ctx: &AssistContext<'_, '_>,
    make: &SyntaxFactory,
    mut arg: ast::Expr,
) -> ast::Expr {
    if let Some(ty) = ctx.sema.type_of_expr(&arg) {
        let adjusted = ty.adjusted();
        if adjusted.strip_reference() != adjusted {
            arg = if let ast::Expr::RefExpr(ref_expr) = &arg
                && let Some(inner) = ref_expr.expr()
            {
                inner
            } else {
                make.expr_prefix(T![*], arg).into()
            };
        }
    }
    arg
}

/// Extract the arithmetic expression to rewrite at the cursor.
fn parse_arith_expr(ctx: &AssistContext<'_, '_>) -> Option<ArithExpr> {
    if !ctx.has_empty_selection() {
        return None;
    }

    if let Some((lhs, op, is_assign, rhs)) = parse_binary_op(ctx) {
        let expr: ast::Expr = ast::BinExpr::cast(lhs.syntax().parent()?)?.into();
        return Some(ArithExpr::Binary { expr, lhs, op, is_assign, rhs });
    }

    if let Some((receiver, exponent)) = parse_pow(ctx) {
        let expr: ast::Expr = ast::MethodCallExpr::cast(receiver.syntax().parent()?)?.into();
        return Some(ArithExpr::Pow { expr, receiver, exponent });
    }

    if let Some(operand) = parse_neg(ctx) {
        let expr: ast::Expr = ast::PrefixExpr::cast(operand.syntax().parent()?)?.into();
        return Some(ArithExpr::Neg { expr, operand });
    }

    None
}

/// Extract the operands of a binary arithmetic expression (e.g. `1 + 2`).
fn parse_binary_op(ctx: &AssistContext<'_, '_>) -> Option<(ast::Expr, ArithOp, bool, ast::Expr)> {
    let expr = ctx.find_node_at_offset::<ast::BinExpr>()?;

    let (op, is_assign) = match expr.op_kind()? {
        BinaryOp::ArithOp(arith_op) => (arith_op, false),
        BinaryOp::Assignment { op: Some(op) } => (op, true),
        _ => return None,
    };
    if !matches!(op, ArithOp::Add | ArithOp::Sub | ArithOp::Mul | ArithOp::Div) {
        return None;
    }

    let lhs = expr.lhs()?;
    let rhs = expr.rhs()?;

    Some((lhs, op, is_assign, rhs))
}

/// Extract the receiver and exponent of a `.pow()` call (e.g. `x.pow(2)`).
fn parse_pow(ctx: &AssistContext<'_, '_>) -> Option<(ast::Expr, ast::Expr)> {
    let expr = ctx.find_node_at_offset::<ast::MethodCallExpr>()?;
    if expr.name_ref()?.text() != "pow" {
        return None;
    }

    let receiver = expr.receiver()?;
    let mut args = expr.arg_list()?.args();
    let exponent = args.next()?;
    if args.next().is_some() {
        return None;
    }

    Some((receiver, exponent))
}

/// Extract the operand of a unary negation (e.g. `-x`).
fn parse_neg(ctx: &AssistContext<'_, '_>) -> Option<ast::Expr> {
    let expr = ctx.find_node_at_offset::<ast::PrefixExpr>()?;
    if expr.op_kind()? != UnaryOp::Neg {
        return None;
    }

    expr.expr()
}

pub(crate) enum ArithKind {
    Saturating,
    Wrapping,
    Checked,
    Strict,
}

impl ArithKind {
    fn assist_id(&self) -> AssistId {
        let s = match self {
            ArithKind::Saturating => "replace_arith_with_saturating",
            ArithKind::Checked => "replace_arith_with_checked",
            ArithKind::Wrapping => "replace_arith_with_wrapping",
            ArithKind::Strict => "replace_arith_with_strict",
        };

        AssistId::refactor_rewrite(s)
    }

    fn label(&self) -> &'static str {
        match self {
            ArithKind::Saturating => "Replace arithmetic with call to saturating_*",
            ArithKind::Checked => "Replace arithmetic with call to checked_*",
            ArithKind::Wrapping => "Replace arithmetic with call to wrapping_*",
            ArithKind::Strict => "Replace arithmetic with call to strict_*",
        }
    }

    fn method_name(&self, arith: &ArithExpr) -> String {
        let prefix = match self {
            ArithKind::Checked => "checked_",
            ArithKind::Wrapping => "wrapping_",
            ArithKind::Saturating => "saturating_",
            ArithKind::Strict => "strict_",
        };

        format!("{prefix}{}", arith.method_suffix())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn arith_kind_method_name() {
        let make = SyntaxFactory::default();
        let unit = make.expr_unit();
        let binary = ArithExpr::Binary {
            expr: unit.clone(),
            lhs: unit.clone(),
            op: ArithOp::Add,
            is_assign: false,
            rhs: unit.clone(),
        };
        let pow =
            ArithExpr::Pow { expr: unit.clone(), receiver: unit.clone(), exponent: unit.clone() };
        let neg = ArithExpr::Neg { expr: unit.clone(), operand: unit.clone() };

        assert_eq!(ArithKind::Saturating.method_name(&binary), "saturating_add");
        assert_eq!(ArithKind::Checked.method_name(&binary), "checked_add");
        assert_eq!(ArithKind::Wrapping.method_name(&pow), "wrapping_pow");
        assert_eq!(ArithKind::Checked.method_name(&neg), "checked_neg");
        assert_eq!(ArithKind::Strict.method_name(&neg), "strict_neg");
    }

    #[test]
    fn replace_arith_with_checked_add() {
        check_assist(
            replace_arith_with_checked,
            r#"
//- minicore: add, builtin_impls
fn main() {
    let x = 1 $0+ 2;
}
"#,
            r#"
fn main() {
    let x = 1.checked_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_saturating_add() {
        check_assist(
            replace_arith_with_saturating,
            r#"
//- minicore: add, builtin_impls
fn main() {
    let x = 1 $0+ 2;
}
"#,
            r#"
fn main() {
    let x = 1.saturating_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_strict_add() {
        check_assist(
            replace_arith_with_strict,
            r#"
fn main() {
    let x = 1 $0+ 2;
}
"#,
            r#"
fn main() {
    let x = 1.strict_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_add() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
//- minicore: add, builtin_impls
fn main() {
    let x = 1 $0+ 2;
}
"#,
            r#"
fn main() {
    let x = 1.wrapping_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_add_add_parenthesis() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
//- minicore: add, builtin_impls
fn main() {
    let x = 1*3 $0+ 2;
}
"#,
            r#"
fn main() {
    let x = (1*3).wrapping_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_add_assign() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
//- minicore: add, builtin_impls
fn main() {
    let mut x = 1;
    x $0+= 2;
}
"#,
            r#"
fn main() {
    let mut x = 1;
    x = x.wrapping_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_add_ref() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = &1;
    x $0+ 2;
}
"#,
            r#"
fn main() {
    let x = &1;
    x.wrapping_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_add_remove_ref() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    1 $0+ &2;
}
"#,
            r#"
fn main() {
    1.wrapping_add(2);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_add_deref() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = &2
    1 $0+ x;
}
"#,
            r#"
fn main() {
    let x = &2
    1.wrapping_add(*x);
}
"#,
        )
    }

    #[test]
    fn replace_arith_not_applicable_with_non_empty_selection() {
        check_assist_not_applicable(
            replace_arith_with_checked,
            r#"
//- minicore: add, builtin_impls
fn main() {
    let x = 1 $0+$0 2;
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_pow() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = 2i32;
    x.$0pow(4u32);
}
"#,
            r#"
fn main() {
    let x = 2i32;
    x.wrapping_pow(4u32);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_checked_pow() {
        check_assist(
            replace_arith_with_checked,
            r#"
fn main() {
    let x = 2i32;
    x.$0pow(4u32);
}
"#,
            r#"
fn main() {
    let x = 2i32;
    x.checked_pow(4u32);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_pow_ref() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = 2i32;
    x.$0pow(&4u32);
}
"#,
            r#"
fn main() {
    let x = 2i32;
    x.wrapping_pow(4u32);
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_neg() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = 2i32;
    let y = -$0x;
}
"#,
            r#"
fn main() {
    let x = 2i32;
    let y = x.wrapping_neg();
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_saturating_neg() {
        check_assist(
            replace_arith_with_saturating,
            r#"
fn main() {
    let x = 2i32;
    let y = -$0x;
}
"#,
            r#"
fn main() {
    let x = 2i32;
    let y = x.saturating_neg();
}
"#,
        )
    }

    #[test]
    fn replace_arith_with_wrapping_neg_needs_parentheses() {
        check_assist(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = 2i32;
    let y = $0--x;
}
"#,
            r#"
fn main() {
    let x = 2i32;
    let y = (-x).wrapping_neg();
}
"#,
        )
    }

    #[test]
    fn replace_arith_not_applicable_on_pow_with_multiple_args() {
        check_assist_not_applicable(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = 2i32;
    x.$0pow(4u32, 3u32);
}
"#,
        )
    }

    #[test]
    fn replace_arith_not_applicable_on_non_neg_prefix() {
        check_assist_not_applicable(
            replace_arith_with_wrapping,
            r#"
fn main() {
    let x = true;
    !$0x;
}
"#,
        )
    }
}
