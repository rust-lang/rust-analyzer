use hir::HirDisplay;
use ide_db::Severity;

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

// Diagnostic: const-arg-has-wrong-type
//
// This diagnostic is triggered when a const generic argument has the wrong type.
pub(crate) fn const_arg_has_wrong_type<'db>(
    ctx: &DiagnosticsContext<'_, 'db>,
    d: &hir::ConstArgHasWrongType<'db>,
) -> Diagnostic {
    let ct = &d.ct;
    let expected_ty = d.expected_ty.display(ctx.db(), ctx.display_target);
    let message = format!("the constant `{ct}` is not of type `{expected_ty}`");
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::Ra("const-arg-has-wrong-type", Severity::Error),
        message,
        d.span.map(Into::into),
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn const_generic_array_length_wrong_type() {
        check_diagnostics(
            r#"
//- minicore: index, slice
struct Struct<const N: i64>(pub [u8; N]);
//                                   ^ error: the constant `N` is not of type `usize`

struct Literal([u8; true]);
//                  ^^^^ error: the constant `true` is not of type `usize`

const C: [u8; true] = loop {};
//            ^^^^ error: the constant `true` is not of type `usize`

static S: [u8; false] = loop {};
//             ^^^^^ error: the constant `false` is not of type `usize`

pub fn function(value: Struct<3>) -> u8 {
    value.0[0]
}

fn main() {}
            "#,
        );
    }

    #[test]
    fn valid_array_index() {
        check_diagnostics(
            r#"
//- minicore: index, slice
type Usize = usize;
struct Struct<const N: Usize>([u8; N]);

fn f(v: [u8; 3]) -> u8 {
    v[0usize]
}

fn main() {}
            "#,
        );
    }
}
