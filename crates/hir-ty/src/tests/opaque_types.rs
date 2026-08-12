use expect_test::expect;

use crate::tests::check_infer;

use super::{check_infer_with_mismatches, check_no_mismatches, check_types};

#[test]
fn associated_type_impl_trait() {
    check_types(
        r#"
trait Foo {}
struct S1;
impl Foo for S1 {}

trait Bar {
    type Item;
    fn bar(&self) -> Self::Item;
}
struct S2;
impl Bar for S2 {
    type Item = impl Foo;
    fn bar(&self) -> Self::Item {
        S1
    }
}

fn test() {
    let x = S2.bar();
      //^ impl Foo + ?Sized
}
        "#,
    );
}

#[test]
fn regression_23125_atpit_with_method_generics() {
    check_no_mismatches(
        r#"
trait Foo {
    type Item;
}
struct S<T>;
struct S2;
impl Foo for S2 {
    type Item = impl Sized;
    fn foo<T>(_: T) -> Self::Item {
        S::<T>
    }
}
"#,
    );
}

// added multi-method non-defining test
#[test]
fn regression_23125_atpit_method_local_generic_non_defining_use() {
    check_no_mismatches(
        r#"
trait Foo {
    type Item;
}
struct S<T>;
struct S2;
impl Foo for S2 {
    type Item = impl Sized;
    fn foo<U>(_: U) -> Self::Item {
        S::<U>
    }
    fn bar() -> Self::Item {
        S::<()>
    }
}
"#,
    );
}

#[test]
fn regression_23125_atpit_with_method_generics_impl_generics() {
    check_no_mismatches(
        r#"
trait Foo {
    type Item;
}
struct S<T>;
struct S2<T>(T);
impl<T> Foo for S2<T> {
    type Item = impl Sized;
    fn foo<U>(_: U) -> Self::Item {
        S::<U>
    }
}
"#,
    );
}

// added impl-generic no-mismatch test
#[test]
fn regression_23125_atpit_impl_generics_are_defining_use() {
    check_no_mismatches(
        r#"
trait Foo {
    type Item;
}
struct S<T>;
struct S2<T>(T);
impl<T> Foo for S2<T> {
    type Item = impl Sized;
    fn foo() -> Self::Item {
        S::<T>
    }
}
"#,
    );
}

#[test]
fn associated_type_impl_traits_complex() {
    check_types(
        r#"
struct Unary<T>(T);
struct Binary<T, U>(T, U);

trait Foo {}
struct S1;
impl Foo for S1 {}

trait Bar {
    type Item;
    fn bar(&self) -> Unary<Self::Item>;
}
struct S2;
impl Bar for S2 {
    type Item = Unary<impl Foo>;
    fn bar(&self) -> Unary<<Self as Bar>::Item> {
        Unary(Unary(S1))
    }
}

trait Baz {
    type Target1;
    type Target2;
    fn baz(&self) -> Binary<Self::Target1, Self::Target2>;
}
struct S3;
impl Baz for S3 {
    type Target1 = impl Foo;
    type Target2 = Unary<impl Bar>;
    fn baz(&self) -> Binary<Self::Target1, Self::Target2> {
        Binary(S1, Unary(S2))
    }
}

fn test() {
    let x = S3.baz();
      //^ Binary<impl Foo + ?Sized, Unary<impl Bar + ?Sized>>
    let y = x.1.0.bar();
      //^ Unary<<impl Bar + ?Sized as Bar>::Item>
}
        "#,
    );
}

#[test]
fn rpit_with_fn_generics_is_defining_use() {
    check_infer(
        r#"
trait Foo {}
struct S;
impl Foo for S {}
struct Wrap<T>(T);
impl<T> Foo for Wrap<T> {}

fn foo<T>() -> impl Foo {
    Wrap(T)
}
"#,
        expect![[r#"
            112..127 '{     Wrap(T) }': Wrap<{unknown}>
            118..122 'Wrap': fn Wrap<{unknown}>({unknown}) -> Wrap<{unknown}>
            118..125 'Wrap(T)': Wrap<{unknown}>
            123..124 'T': {unknown}
        "#]],
    );
}

#[test]
fn associated_type_with_impl_trait_in_tuple() {
    check_no_mismatches(
        r#"
pub trait Iterator {
    type Item;
}

pub trait Value {}

fn bar<I: Iterator<Item = (usize, impl Value)>>() {}

fn foo() {
    bar();
}
"#,
    );
}

#[test]
fn associated_type_with_impl_trait_in_nested_tuple() {
    check_no_mismatches(
        r#"
pub trait Iterator {
    type Item;
}

pub trait Value {}

fn bar<I: Iterator<Item = ((impl Value, usize), u32)>>() {}

fn foo() {
    bar();
}
"#,
    );
}

#[ignore = "FIXME(next-solver): TAIT support was removed, need to rework it to work with `#[define_opaque]`"]
#[test]
fn type_alias_impl_trait_simple() {
    check_no_mismatches(
        r#"
trait Trait {}

struct Struct;

impl Trait for Struct {}

type AliasTy = impl Trait;

static ALIAS: AliasTy = {
    let res: AliasTy = Struct;
    res
};
"#,
    );

    check_infer_with_mismatches(
        r#"
trait Trait {}

struct Struct;

impl Trait for Struct {}

type AliasTy = impl Trait;

static ALIAS: i32 = {
    // TATIs cannot be define-used if not in signature or type annotations
    let _a: AliasTy = Struct;
    5
};
"#,
        expect![[r#"
            106..220 '{     ...   5 }': i32
            191..193 '_a': impl Trait + ?Sized
            205..211 'Struct': Struct
            217..218 '5': i32
            205..211: expected impl Trait + ?Sized, got Struct
        "#]],
    )
}

#[test]
fn leak_auto_traits() {
    check_no_mismatches(
        r#"
//- minicore: send
fn foo() -> impl Sized {}

fn is_send<T: Send>(_: T) {}

fn main() {
    is_send(foo());
}
        "#,
    );
}

#[test]
fn regression_21455() {
    check_infer(
        r#"
//- minicore: copy

struct Vec<T>(T);
impl<T> Vec<T> {
    pub fn new() -> Self { loop {} }
}

pub struct Miku {}

impl Miku {
    pub fn all_paths_to(&self) -> impl Copy {
        Miku {
            full_paths: Vec::new(),
        }
    }
}
    "#,
        expect![[r#"
            61..72 '{ loop {} }': Vec<T>
            63..70 'loop {}': !
            68..70 '{}': ()
            133..137 'self': &'? Miku
            152..220 '{     ...     }': Miku
            162..214 'Miku {...     }': Miku
            193..201 'Vec::new': fn new<{unknown}>() -> Vec<{unknown}>
            193..203 'Vec::new()': Vec<{unknown}>
        "#]],
    );
}
