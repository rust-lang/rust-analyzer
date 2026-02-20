use expect_test::expect;

use crate::tests::{check, check_edit_with_trigger_character, check_with_trigger_character};

#[test]
fn only_param() {
    check(
        r#"
fn foo(file_id: usize) {}
fn bar(file_id: usize) {}
fn baz(file$0) {}
"#,
        expect![[r#"
            bn file_id: usize
            kw mut
            kw ref
        "#]],
    );
}

#[test]
fn last_param() {
    check(
        r#"
fn foo(file_id: usize) {}
fn bar(file_id: usize) {}
fn baz(foo: (), file$0) {}
"#,
        expect![[r#"
            bn file_id: usize
            kw mut
            kw ref
        "#]],
    );
}

#[test]
fn first_param() {
    check(
        r#"
fn foo(file_id: usize) {}
fn bar(file_id: usize) {}
fn baz(file$0 id: u32) {}
"#,
        expect![[r#"
            bn file_id: usize,
            kw mut
            kw ref
        "#]],
    );
}

#[test]
fn repeated_param_name() {
    check(
        r#"
fn foo(file_id: usize) {}
fn bar(file_id: u32, $0) {}
"#,
        expect![[r#"
            kw mut
            kw ref
        "#]],
    );

    check(
        r#"
fn f(#[foo = "bar"] baz: u32,) {}
fn g(baz: (), ba$0)
"#,
        expect![[r#"
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn trait_param() {
    check(
        r#"
pub(crate) trait SourceRoot {
    pub fn contains(file_id: usize) -> bool;
    pub fn syntax(file$0)
}
"#,
        expect![[r#"
            bn &mut self
            bn &self
            bn file_id: usize
            bn mut self
            bn self
            kw mut
            kw ref
        "#]],
    );
}

#[test]
fn in_inner_function() {
    check(
        r#"
fn outer(text: &str) {
    fn inner($0)
}
"#,
        expect![[r#"
            bn text: &str
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn trigger_by_l_paren() {
    check_with_trigger_character(
        r#"
fn foo($0)
"#,
        Some('('),
        expect![[]],
    )
}

#[test]
fn shows_non_ident_pat_param() {
    check(
        r#"
struct Bar { bar: u32 }
fn foo(Bar { bar }: Bar) {}
fn foo2($0) {}
"#,
        expect![[r#"
            st Bar
            bn Bar { bar }: Bar
            bn Bar {…} Bar { bar$1 }: Bar$0
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn in_impl_only_param() {
    check(
        r#"
struct A {}

impl A {
    fn foo(file_id: usize) {}
    fn new($0) {}
}
"#,
        expect![[r#"
            sp Self
            st A
            bn &mut self
            bn &self
            bn file_id: usize
            bn mut self
            bn self
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn in_impl_after_self() {
    check(
        r#"
struct A {}

impl A {
    fn foo(file_id: usize) {}
    fn new(self, $0) {}
}
"#,
        expect![[r#"
            sp Self
            st A
            bn file_id: usize
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn in_trait_only_param() {
    check(
        r#"
trait A {
    fn foo(file_id: usize) {}
    fn new($0) {}
}
"#,
        expect![[r#"
            bn &mut self
            bn &self
            bn file_id: usize
            bn mut self
            bn self
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn in_trait_after_self() {
    check(
        r#"
trait A {
    fn foo(file_id: usize) {}
    fn new(self, $0) {}
}
"#,
        expect![[r#"
            bn file_id: usize
            kw mut
            kw ref
        "#]],
    )
}

// doesn't complete qux due to there being no expression after
// see source_analyzer::adjust comment
#[test]
fn local_fn_shows_locals_for_params() {
    check(
        r#"
fn outer() {
    let foo = 3;
    {
        let bar = 3;
        fn inner($0) {}
        let baz = 3;
        let qux = 3;
    }
    let fez = 3;
}
"#,
        expect![[r#"
            bn bar: i32
            bn baz: i32
            bn foo: i32
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn closure_shows_locals_for_params() {
    check(
        r#"
fn outer() {
    let foo = 3;
    {
        let bar = 3;
        |$0| {};
        let baz = 3;
        let qux = 3;
    }
    let fez = 3;
}
"#,
        expect![[r#"
            bn bar: i32
            bn baz: i32
            bn foo: i32
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn completes_fully_equal() {
    check(
        r#"
fn foo(bar: u32) {}
fn bar(bar$0) {}
"#,
        expect![[r#"
            bn bar: u32
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn not_shows_fully_equal_inside_pattern_params() {
    check(
        r#"
fn foo(bar: u32) {}
fn bar((a, bar$0)) {}
"#,
        expect![[r#"
            kw mut
            kw ref
        "#]],
    )
}

#[test]
fn not_shows_locals_inside_pattern_params() {
    check(
        r#"
fn outer() {
    let foo = 3;
    {
        let bar = 3;
        |($0)| {};
        let baz = 3;
        let qux = 3;
    }
    let fez = 3;
}
"#,
        expect![[r#"
            kw mut
            kw ref
        "#]],
    );
    check(
        r#"
fn outer() {
    let foo = 3;
    {
        let bar = 3;
        fn inner(($0)) {}
        let baz = 3;
        let qux = 3;
    }
    let fez = 3;
}
"#,
        expect![[r#"
            kw mut
            kw ref
        "#]],
    );
}

#[test]
fn completes_for_params_with_attributes() {
    check(
        r#"
fn f(foo: (), #[baz = "qux"] mut bar: u32) {}
fn g(foo: (), #[baz = "qux"] mut ba$0)
"#,
        expect![[r##"
            bn #[baz = "qux"] mut bar: u32
        "##]],
    )
}

#[test]
fn closure_within_param_fn_single() {
    check_edit_with_trigger_character(
        "|_| ",
        r#"
//- minicore: fn
fn foo(f: impl Fn(u32)) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo(f: impl Fn(u32)) {}
fn main() {
    foo(|${1:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_fn_multiple() {
    check_edit_with_trigger_character(
        "|_, _| ",
        r#"
//- minicore: fn
fn foo(f: impl Fn(u32, i64) -> bool) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo(f: impl Fn(u32, i64) -> bool) {}
fn main() {
    foo(|${1:_}, ${2:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_fn_mut() {
    check_edit_with_trigger_character(
        "|_| ",
        r#"
//- minicore: fn
fn foo(f: impl FnMut(u32)) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo(f: impl FnMut(u32)) {}
fn main() {
    foo(|${1:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_fn_once() {
    check_edit_with_trigger_character(
        "|_| ",
        r#"
//- minicore: fn
fn foo(f: impl FnOnce(String) -> String) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo(f: impl FnOnce(String) -> String) {}
fn main() {
    foo(|${1:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_second_arg() {
    check_edit_with_trigger_character(
        "|_| ",
        r#"
//- minicore: fn
fn foo(x: u32, f: impl Fn(bool) -> bool) {}
fn main() {
    foo(0, |$0);
}
"#,
        r#"
fn foo(x: u32, f: impl Fn(bool) -> bool) {}
fn main() {
    foo(0, |${1:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_method_call() {
    check_edit_with_trigger_character(
        "|_| ",
        r#"
//- minicore: fn
struct S;
impl S {
    fn foo(&self, f: impl FnOnce(u32) -> bool) {}
}
fn main() {
    S.foo(|$0);
}
"#,
        r#"
struct S;
impl S {
    fn foo(&self, f: impl FnOnce(u32) -> bool) {}
}
fn main() {
    S.foo(|${1:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_method_second_closure_arg() {
    check_edit_with_trigger_character(
        "|_, _| ",
        r#"
//- minicore: fn
struct S;
impl S {
    fn foo(&self, a: impl Fn(u32), b: impl Fn(bool, i64)) {}
}
fn main() {
    S.foo(|_| {}, |$0);
}
"#,
        r#"
struct S;
impl S {
    fn foo(&self, a: impl Fn(u32), b: impl Fn(bool, i64)) {}
}
fn main() {
    S.foo(|_| {}, |${1:_}, ${2:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_where_clause() {
    check_edit_with_trigger_character(
        "|_, _| ",
        r#"
//- minicore: fn
fn foo<F>(f: F) where F: Fn(u32, bool) -> bool {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo<F>(f: F) where F: Fn(u32, bool) -> bool {}
fn main() {
    foo(|${1:_}, ${2:_}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_generic_single() {
    check_edit_with_trigger_character(
        "|_: T| ",
        r#"
//- minicore: fn
fn foo<T>(f: impl Fn(T) -> T) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo<T>(f: impl Fn(T) -> T) {}
fn main() {
    foo(|${1:_}: ${2:T}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_generic_mixed() {
    check_edit_with_trigger_character(
        "|_, _: T| ",
        r#"
//- minicore: fn
fn foo<T>(f: impl Fn(u32, T) -> bool) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo<T>(f: impl Fn(u32, T) -> bool) {}
fn main() {
    foo(|${1:_}, ${2:_}: ${3:T}| $0);
}
"#,
        Some('|'),
    );
}

#[test]
fn closure_within_param_generic_multiple() {
    check_edit_with_trigger_character(
        "|_: T, _: U| ",
        r#"
//- minicore: fn
fn foo<T, U>(f: impl Fn(T, U)) {}
fn main() {
    foo(|$0);
}
"#,
        r#"
fn foo<T, U>(f: impl Fn(T, U)) {}
fn main() {
    foo(|${1:_}: ${2:T}, ${3:_}: ${4:U}| $0);
}
"#,
        Some('|'),
    );
}
