use either::Either;
use syntax::{AstNode, ast};

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext, adjusted_display_range};

// Diagnostic: explicit-destructor-call
//
// This diagnostic is triggered if a destructor method is called explicitly.
pub(crate) fn explicit_destructor_call(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::ExplicitDestructorCall,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0040"),
        "explicit use of destructor method",
        adjusted_display_range(ctx, d.expr, &|node| {
            let Either::Left(expr) = node else { return None };
            match expr {
                ast::Expr::MethodCallExpr(it) => it.name_ref()?.syntax().text_range().into(),
                _ => None,
            }
        }),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn method_call() {
        check_diagnostics(
            r#"
//- minicore: drop
struct Foo;

impl Drop for Foo {
    fn drop(&mut self) {}
}

fn demo() {
    let mut x = Foo;
    x.drop();
    //^^^^ error: explicit use of destructor method
}
"#,
        );
    }

    #[test]
    fn std_mem_drop() {
        check_diagnostics(
            r#"
//- minicore: drop
struct Foo;

impl Drop for Foo {
    fn drop(&mut self) {}
}

fn demo() {
    let x = Foo;
    drop(x);
}
"#,
        );
    }

    #[test]
    fn inherent_drop_method() {
        check_diagnostics(
            r#"
struct Foo;

impl Foo {
    fn drop(&mut self) {}
}

fn demo() {
    let mut x = Foo;
    x.drop();
}
"#,
        );
    }

    #[test]
    fn custom_trait_drop_method() {
        check_diagnostics(
            r#"
trait MyDrop {
    fn drop(&mut self);
}

struct Foo;

impl MyDrop for Foo {
    fn drop(&mut self) {}
}

fn demo() {
    let mut x = Foo;
    x.drop();
}
"#,
        );
    }
}
