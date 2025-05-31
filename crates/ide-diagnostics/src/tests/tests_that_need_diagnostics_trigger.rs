use super::{check_diagnostics, check_diagnostics_with_disabled};

#[test]
fn recursive_const_should_not_panic() {
    check_diagnostics_with_disabled(
        r#"
struct Foo<const N: usize> {}
impl <const N: Foo<N
"#,
        &["syntax-error"],
    );

    check_diagnostics(
        r#"
struct Foo<const N: usize>;
const N: Foo<N> = Foo;
"#,
    );
}
