use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: dyn-compatible
pub(crate) fn dyn_compatible<'db>(
    ctx: &DiagnosticsContext<'_, 'db>,
    d: &hir::DynCompatible,
) -> Diagnostic {
    let trait_ = hir::Trait::from(d.trait_def_id.0);
    let trait_name = trait_.name(ctx.db());

    let message =
        format!("the trait `{}` is not dyn compatible", trait_name.display(ctx.db(), ctx.edition),);
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::RustcHardError("E0038"),
        message,
        d.span.map(Into::into),
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn dyn_incompatible_trait_is_reported() {
        check_diagnostics(
            r#"
trait NotDynSafe {
    fn foo(&self) -> Self;
}

fn mk() -> &'static dyn NotDynSafe { loop {} }

fn f() {
    let _: &dyn NotDynSafe = loop {};
        // ^^^^^^^^^^^^^^^ error: the trait `NotDynSafe` is not dyn compatible
}

fn g() {
    mk();
  //^^^^ error: the trait `NotDynSafe` is not dyn compatible
}
"#,
        );
    }

    #[test]
    fn dyn_compatible_trait_is_not_reported() {
        check_diagnostics(
            r#"
//- minicore: send
fn f() {
    let _: &dyn Send = loop {};
}
"#,
        );
    }

    #[test]
    fn dyn_incompatible_assoc_binding_is_reported() {
        check_diagnostics(
            r#"
trait NotDynSafe { fn foo(&self) -> Self; }
trait Outer { type Item; }
fn f() {
    let _: &dyn Outer<Item = dyn NotDynSafe> = loop {};
        // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error: the trait `NotDynSafe` is not dyn compatible
}
"#,
        );
    }

    #[test]
    fn dyn_trait_with_unsized_generic_arg_is_accepted() {
        check_diagnostics(
            r#"
trait Trait<T: Sized> {}
fn f() {
    let _: &dyn Trait<[u32]> = loop {};
}
"#,
        );
    }

    #[test]
    fn issue_23475_no_false_positive_for_coerced_tuple() {
        check_diagnostics(
            r#"
//- minicore: coerce_unsized, send
use core::{marker::Unsize, ops::CoerceUnsized};

struct Box<T: ?Sized>(*const T);
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Box<U>> for Box<T> {}
impl<T> Box<T> {
    fn new(t: T) -> Self { loop {} }
}

fn foo<T>(x: (T, ())) -> Box<T> {
    Box::new(x.0)
}

fn main() {
    let _: Box<dyn Send> = foo((match () { () => () }, ()));
}
"#,
        );
    }

    #[test]
    fn unsized_array_element_is_reported() {
        check_diagnostics(
            r#"
//- minicore: sized
fn g() {
    let _: [str; 1] = loop {};
        // ^^^^^^^^ error: the trait bound `str: Sized` is not satisfied
}
"#,
        );
    }

    #[test]
    fn nested_dyn_through_adt_generic_arg_is_reported() {
        check_diagnostics(
            r#"
//- minicore: sized
trait NotDynSafe { fn foo(&self) -> Self; }
struct Wrapper<T>(T);
fn f() {
    let _: Wrapper<&dyn NotDynSafe> = loop {};
        // ^^^^^^^^^^^^^^^^^^^^^^^^ error: the trait `NotDynSafe` is not dyn compatible
}
"#,
        );
    }

    #[test]
    fn dyn_in_trait_signature_is_reported() {
        check_diagnostics(
            r#"
trait NotDynSafe { fn foo(&self) -> Self; }
fn mk() -> &'static dyn NotDynSafe { loop {} }
fn closure_upvar() {
    let d = mk();
          //^^^^ error: the trait `NotDynSafe` is not dyn compatible
    let _ = move || d;
}
fn via_generic_call() {
    take(mk);
       //^^ error: the trait `NotDynSafe` is not dyn compatible
}
fn take<T>(_: T) {}
"#,
        );
    }

    #[test]
    fn string_literal_array_length_is_silent_in_ra() {
        check_diagnostics(
            r#"
//- minicore: sized
fn c() {
    let _: [u8; "a"] = loop {};
}
"#,
        );
    }
}
