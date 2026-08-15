use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: unresolved-record-expr
//
// This diagnostic is triggered if a struct expression's path does not resolve to an existing struct, union, or enum variant
pub(crate) fn unresolved_record_expr(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::UnresolvedRecordExpr,
) -> Diagnostic {
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::RustcHardError("E0422"),
        format!(
            "cannot find struct, variant or union type `{}` in this scope",
            d.name.display(ctx.sema.db, ctx.edition)
        ),
        d.expr.map(|it| it.into()),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn reports_unresolved_record_expr() {
        check_diagnostics(
            r#"
//- /main.rs crate:main
struct Point { x: i32, y: i32 }

fn main() {
    let p = Poin { x: 1, y: 2 };
          //^^^^^^^^^^^^^^^^^^^ error: cannot find struct, variant or union type `Poin` in this scope
}
"#,
        );
    }
}
