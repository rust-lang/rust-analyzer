use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) fn ambiguous_module(
    ctx: &DiagnosticsContext<'_, '_>,
    d: &hir::AmbiguousModule,
) -> Diagnostic {
    let [first, second] = &d.candidates;
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::RustcHardError("E0761"),
        format!("file for module found at both `{first}` and `{second}`"),
        d.decl.map(|it| it.into()),
    )
    .stable()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics_with_disabled;

    #[test]
    fn ambiguous_module() {
        check_diagnostics_with_disabled(
            r#"
//- /lib.rs
  mod foo;
//^^^^^^^^ error: file for module found at both `foo.rs` and `foo/mod.rs`
//- /foo.rs
//- /foo/mod.rs
"#,
            &["unlinked-file"],
        );
    }

    #[test]
    fn ambiguous_module_in_inline_module() {
        check_diagnostics_with_disabled(
            r#"
//- /lib.rs
mod outer {
    mod foo;
  //^^^^^^^^ error: file for module found at both `outer/foo.rs` and `outer/foo/mod.rs`
}
//- /outer/foo.rs
//- /outer/foo/mod.rs
"#,
            &["unlinked-file"],
        );
    }

    #[test]
    fn ambiguous_module_in_file_module() {
        check_diagnostics_with_disabled(
            r#"
//- /lib.rs
mod outer;
//- /outer.rs
  mod foo;
//^^^^^^^^ error: file for module found at both `outer/foo.rs` and `outer/foo/mod.rs`
//- /outer/foo.rs
//- /outer/foo/mod.rs
"#,
            &["unlinked-file"],
        );
    }

    #[test]
    fn ambiguous_module_with_path_attribute() {
        check_diagnostics_with_disabled(
            r#"
//- /lib.rs
#[path = "foo/mod.rs"]
mod foo;
use foo::FromDirectory;
//- /foo.rs
pub struct FromFile;
//- /foo/mod.rs
pub struct FromDirectory;
"#,
            &["unlinked-file"],
        );
    }

    #[test]
    fn ambiguous_module_disabled_by_cfg() {
        check_diagnostics_with_disabled(
            r#"
//- /lib.rs
#[cfg(FALSE)]
mod foo;
//- /foo.rs
//- /foo/mod.rs
"#,
            &["unlinked-file", "inactive_code"],
        );
    }
}
