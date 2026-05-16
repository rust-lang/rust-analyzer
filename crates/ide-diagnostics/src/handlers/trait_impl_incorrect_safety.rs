use hir::InFile;
use syntax::AstNode;

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext, adjusted_display_range};

// Diagnostic: unsafe-inherent-impl
//
// This diagnostic is triggered when an inherent implementation is marked `unsafe`
pub(crate) fn unsafe_inherent_impl(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::UnsafeInherentImpl,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0197"),
        "inherent impls cannot be unsafe",
        adjusted_display_range(ctx, InFile { file_id: d.file_id, value: d.impl_ }, &|impl_| {
            Some((impl_.unsafe_token()?.text_range()).cover(impl_.self_ty()?.syntax().text_range()))
        }),
    )
    .stable()
}

// Diagnostic: unsafe-negative-impl
//
// This diagnostic is triggered when a negative implementation is marked `unsafe`
pub(crate) fn unsafe_negative_impl(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::UnsafeNegativeImpl,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0198"),
        "negative impls cannot be unsafe",
        adjusted_display_range(ctx, InFile { file_id: d.file_id, value: d.impl_ }, &|impl_| {
            Some((impl_.unsafe_token()?.text_range()).cover(impl_.self_ty()?.syntax().text_range()))
        }),
    )
    .stable()
}

// Diagnostic: unsafe-impl-of-safe-trait
//
// This diagnostic is triggered when an implementation of a safe trait is marked `unsafe`
pub(crate) fn unsafe_impl_of_safe_trait(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::UnsafeImplOfSafeTrait,
) -> Diagnostic {
    let trait_name = d.trait_.name(ctx.db());
    let trait_name = trait_name.display(ctx.db(), ctx.edition);

    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0199"),
        format!("implementing the trait `{trait_name}` is not unsafe"),
        adjusted_display_range(ctx, InFile { file_id: d.file_id, value: d.impl_ }, &|impl_| {
            Some(impl_.unsafe_token()?.text_range().cover(impl_.self_ty()?.syntax().text_range()))
        }),
    )
    .stable()
}

// Diagnostic: safe-impl-of-unsafe-trait
//
// This diagnostic is triggered when an implementation of an unsafe trait is missing `unsafe`
pub(crate) fn safe_impl_of_unsafe_trait(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::SafeImplOfUnsafeTrait,
) -> Diagnostic {
    let trait_name = d.trait_.name(ctx.db());
    let trait_name = trait_name.display(ctx.db(), ctx.edition);

    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0200"),
        format!("the trait `{trait_name}` requires an `unsafe impl` declaration"),
        adjusted_display_range(ctx, InFile { file_id: d.file_id, value: d.impl_ }, &|impl_| {
            Some(impl_.impl_token()?.text_range().cover(impl_.self_ty()?.syntax().text_range()))
        }),
    )
    .stable()
}

// Diagnostic: safe-impl-of-dangling-drop
//
// This diagnostic is triggered when an implementation of `Drop` using `#[may_dangle]` is missing `unsafe`
pub(crate) fn safe_impl_of_dangling_drop(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::SafeImplOfDanglingDrop,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::RustcHardError("E0569"),
        "requires an `unsafe impl` declaration due to `#[may_dangle]` attribute",
        adjusted_display_range(ctx, InFile { file_id: d.file_id, value: d.impl_ }, &|impl_| {
            Some(impl_.impl_token()?.text_range().cover(impl_.self_ty()?.syntax().text_range()))
        }),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn simple() {
        check_diagnostics(
            r#"
trait Safe {}
unsafe trait Unsafe {}

  impl Safe for () {}

  impl Unsafe for () {}
//^^^^^^^^^^^^^^^^^^  error: the trait `Unsafe` requires an `unsafe impl` declaration

  unsafe impl Safe for () {}
//^^^^^^^^^^^^^^^^^^^^^^^ error: implementing the trait `Safe` is not unsafe

  unsafe impl Unsafe for () {}
"#,
        );
    }

    #[test]
    fn drop_may_dangle() {
        check_diagnostics(
            r#"
#![feature(lang_items)]
#[lang = "drop"]
trait Drop {}
struct S<T>;
struct L<'l>;

  impl<T> Drop for S<T> {}

  impl<#[may_dangle] T> Drop for S<T> {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error: requires an `unsafe impl` declaration due to `#[may_dangle]` attribute

  unsafe impl<T> Drop for S<T> {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error: implementing the trait `Drop` is not unsafe

  unsafe impl<#[may_dangle] T> Drop for S<T> {}

  impl<'l> Drop for L<'l> {}

  impl<#[may_dangle] 'l> Drop for L<'l> {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error: requires an `unsafe impl` declaration due to `#[may_dangle]` attribute

  unsafe impl<'l> Drop for L<'l> {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error: implementing the trait `Drop` is not unsafe

  unsafe impl<#[may_dangle] 'l> Drop for L<'l> {}
"#,
        );
    }

    #[test]
    fn negative() {
        check_diagnostics(
            r#"
trait Trait {}

  impl !Trait for () {}

  unsafe impl !Trait for () {}
//^^^^^^^^^^^^^^^^^^^^^^^^^ error: negative impls cannot be unsafe

unsafe trait UnsafeTrait {}

  impl !UnsafeTrait for () {}

  unsafe impl !UnsafeTrait for () {}
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ error: negative impls cannot be unsafe

"#,
        );
    }

    #[test]
    fn inherent() {
        check_diagnostics(
            r#"
struct S;

  impl S {}

  unsafe impl S {}
//^^^^^^^^^^^^^ error: inherent impls cannot be unsafe
"#,
        );
    }

    #[test]
    fn unsafe_unresolved_trait() {
        check_diagnostics(
            r#"
unsafe impl TestTrait for u32 {}
        "#,
        );
    }
}
