use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: type-must-be-known
//
// This diagnostic is triggered when rust-analyzer cannot infer some type.
pub(crate) fn type_must_be_known(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::TypeMustBeKnown,
) -> Diagnostic {
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::RustcHardError("E0282"),
        "type annotations needed; type must be known at this point",
        d.at_point.map(|it| it.into()),
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn smoke_test() {
        check_diagnostics(
            r#"
fn foo() {
    let var = loop {};
     // ^^^ 💡 warn: unused variable
    var();
 // ^^^ error: type annotations needed; type must be known at this point
    let var = loop {};
     // ^^^ 💡 warn: unused variable
    var[0];
 // ^^^ error: type annotations needed; type must be known at this point
}
        "#,
        );
    }
}
