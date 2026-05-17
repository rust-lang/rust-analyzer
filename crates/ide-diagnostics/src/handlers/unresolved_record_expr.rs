use either::Either;
use syntax::{AstNode, ast::Expr};

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: unresolved-record-expr
//
// This diagnostic is triggered if the struct, variant, or union type referred to by a record expression does not exist in the current scope.
pub(crate) fn unresolved_record_expr(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::UnresolvedRecordExpr,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0422"),
        "cannot find struct, variant or union type in this scope".to_owned(),
        crate::adjusted_display_range(ctx, d.expr, &|expr| match expr {
            Either::Left(Expr::RecordExpr(it)) => it.path().map(|p| p.syntax().text_range()),
            _ => None,
        }),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn unresolved_record_expr() {
        check_diagnostics(
            r#"
struct Exist {
    x: i32,
    y: i32,
}

fn main() {
    let _ = Exist { x: 1, y: 2 };
    let _ = DoesNotExist { x: 1, y: 2 };
         // ^^^^^^^^^^^^ error: cannot find struct, variant or union type in this scope
}
"#,
        );
    }
}
