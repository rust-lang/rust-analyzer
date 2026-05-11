use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: explicit-destructor-call
//
// This diagnostic is triggered when the method `Drop::drop` is called
// explicitly.  Use `drop` (the free function in `std::mem`) instead.
pub(crate) fn explicit_destructor_call(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::ExplicitDestructorCall,
) -> Diagnostic {
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::RustcHardError("E0040"),
        "explicit use of destructor method",
        d.expr.map(|it| it.into()),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn explicit_drop_call_on_droppable_value() {
        check_diagnostics(
            r#"
//- minicore: drop, sized
struct S;
impl core::ops::Drop for S {
    fn drop(&mut self) {}
}
fn f(mut s: S) {
    s.drop();
  //^^^^^^^^ error: explicit use of destructor method
}
"#,
        );
    }

    #[test]
    fn free_function_drop_is_allowed() {
        check_diagnostics(
            r#"
//- minicore: drop, sized
struct S;
impl core::ops::Drop for S {
    fn drop(&mut self) {}
}
fn f(s: S) {
    drop(s);
}
"#,
        );
    }

    #[test]
    fn inherent_method_named_drop_is_allowed() {
        check_diagnostics(
            r#"
//- minicore: drop, sized
struct S;
impl S {
    fn drop(&self) {}
}
fn f(s: S) {
    s.drop();
}
"#,
        );
    }
}
