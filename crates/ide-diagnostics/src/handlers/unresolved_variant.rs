use either::Either;
use syntax::{AstNode, ast};

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext, adjusted_display_range};

// Diagnostic: unresolved-variant
//
// This diagnostic is triggered if the struct, variant, or union type referred to by a record expression does not exist in the current scope.
pub(crate) fn unresolved_variant(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::UnresolvedVariant,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0422"),
        "cannot find struct, variant or union type in this scope".to_owned(),
        adjusted_display_range(ctx, d.node, &|node| match node {
            Either::Left(ast::Expr::RecordExpr(it)) => it.path().map(|p| p.syntax().text_range()),
            Either::Right(ast::Pat::RecordPat(it)) => it.path().map(|p| p.syntax().text_range()),
            Either::Right(ast::Pat::TupleStructPat(it)) => {
                it.path().map(|p| p.syntax().text_range())
            }
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
fn main() {
    let _ = DoesNotExist { x: 1, y: 2 };
    //      ^^^^^^^^^^^^ error: cannot find struct, variant or union type in this scope
}
"#,
        );
    }

    #[test]
    fn unresolved_record_pat() {
        check_diagnostics(
            r#"
fn main() {
    struct Exist { x: i32, y: i32 }
    let a = Exist { x: 1, y: 2 };
    match a {
        Exist { .. } => {}
        DoesNotExist { .. } => {}
     // ^^^^^^^^^^^^ error: cannot find struct, variant or union type in this scope
        _ => {}
    }
}
"#,
        );
    }

    #[test]
    fn unresolved_tuple_struct_pat() {
        check_diagnostics(
            r#"
fn main() {
    struct Tuple(i32, i32);
    let t = Tuple(1, 2);
    match t {
        Tuple( .. ) => {}
        DoesNotExist( .. ) => {}
    //  ^^^^^^^^^^^^ error: cannot find struct, variant or union type in this scope
        _ => {}
    }
}
"#,
        );
    }
}
