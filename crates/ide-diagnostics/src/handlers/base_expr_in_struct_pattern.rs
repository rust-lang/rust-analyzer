use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: base-expr-in-struct-pattern
//
// This diagnostic is triggered if a struct pattern contains a base expression (`..base`).
pub(crate) fn base_expr_in_struct_pattern(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::BaseExprInStructPattern,
) -> Diagnostic {
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::SyntaxError,
        "base expressions aren't allowed in struct patterns",
        d.node.map(Into::into),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn spread_variable() {
        check_diagnostics(
            r#"
struct Foo { bar: u32, baz: u32 }
fn test(f: Foo, g: Foo) {
    if let Foo { ..g } = f {}
                // ^ error: base expressions aren't allowed in struct patterns
    if let Foo { bar: 0, ..g } = f {}
                        // ^ error: base expressions aren't allowed in struct patterns
    if let Foo { bar: 0, baz: 0, ..g } = f {}
                                // ^ error: base expressions aren't allowed in struct patterns
}
        "#,
        );
    }

    #[test]
    fn spread_default() {
        check_diagnostics(
            r#"
struct Foo { bar: u32, baz: u32 }
fn test(f: Foo) {
    if let Foo { ..Default::default() } = f {}
                // ^^^^^^^^^^^^^^^^^^ error: base expressions aren't allowed in struct patterns
}
        "#,
        );
    }

    #[test]
    fn spread_struct() {
        check_diagnostics(
            r#"
struct Foo { bar: u32, baz: u32 }
fn test(f: Foo) {
    if let Foo { ..Foo { bar: 0, baz: 0 } } = f {}
                // ^^^^^^^^^^^^^^^^^^^^^^ error: base expressions aren't allowed in struct patterns
}
        "#,
        );
    }
}
