use crate::{
    DefWithBodyId, ModuleDefId,
    expr_store::pretty::{print_function, print_struct},
    hir::MatchArm,
    test_db::TestDB,
};
use expect_test::{Expect, expect};
use la_arena::RawIdx;
use test_fixture::WithFixture;

use super::super::*;

fn lower_and_print(#[rust_analyzer::rust_fixture] ra_fixture: &str, expect: Expect) {
    let db = TestDB::with_files(ra_fixture);

    let krate = db.fetch_test_crate();
    let def_map = db.crate_def_map(krate);
    let mut defs = vec![];
    for (_, module) in def_map.modules() {
        for decl in module.scope.declarations() {
            let def: GenericDefId = match decl {
                ModuleDefId::ModuleId(_) => continue,
                ModuleDefId::FunctionId(id) => id.into(),
                ModuleDefId::AdtId(id) => id.into(),
                ModuleDefId::ConstId(id) => id.into(),
                ModuleDefId::StaticId(id) => id.into(),
                ModuleDefId::TraitId(id) => id.into(),
                ModuleDefId::TraitAliasId(id) => id.into(),
                ModuleDefId::TypeAliasId(id) => id.into(),
                ModuleDefId::EnumVariantId(_) => continue,
                ModuleDefId::BuiltinType(_) => continue,
                ModuleDefId::MacroId(_) => continue,
            };
            defs.push(def);
        }
    }

    let mut out = String::new();
    for def in defs {
        match def {
            GenericDefId::AdtId(adt_id) => match adt_id {
                crate::AdtId::StructId(struct_id) => {
                    out += &print_struct(&db, &db.struct_signature(struct_id), Edition::CURRENT);
                }
                crate::AdtId::UnionId(union_id) => todo!(),
                crate::AdtId::EnumId(enum_id) => todo!(),
            },
            GenericDefId::ConstId(const_id) => todo!(),
            GenericDefId::FunctionId(function_id) => {
                out += &print_function(&db, &db.function_signature(function_id), Edition::CURRENT)
            }

            GenericDefId::ImplId(impl_id) => todo!(),
            GenericDefId::StaticId(static_id) => todo!(),
            GenericDefId::TraitAliasId(trait_alias_id) => todo!(),
            GenericDefId::TraitId(trait_id) => todo!(),
            GenericDefId::TypeAliasId(type_alias_id) => todo!(),
        }
    }

    expect.assert_eq(&out);
}

#[test]
fn structs() {
    lower_and_print(
        r"
struct S { field: foo, }
struct S(i32, u32, &'static str);
struct S;

struct S<'a, 'b, T: Clone, const C: usize = 3, X = ()> where X: Default, for<'a, 'c> fn() -> i32: for<'b> Trait<'a, Item = Boo>;
",
        expect![[r#"
            struct S {...}
            struct S(...)
            ;
            struct S;
            struct S<'a, 'b, T, const C: usize = 3, X = ()>
            where
                T: Clone,
                X: Default,
                for<'a, 'c> fn() -> i32: for<'b> Trait::<'a, Item = Boo>
            ;
        "#]],
    );
}

#[test]
fn functions() {
    lower_and_print(
        r"
fn foo<'a, const C: usize = 314235, T: Trait<Item = A> = B>(Struct { foo: bar }: &Struct, _: (), a: u32) -> &'a dyn Fn() -> i32 where (): Default {}

fn impl_trait_args<T>(_: impl Trait) {}
fn impl_trait_args2<T>(_: impl Trait<impl Trait>) {}

fn impl_trait_ret<T>() -> impl Trait {}
fn impl_trait_ret2<T>() -> impl Trait<impl Trait> {}

fn not_allowed1(f: impl Fn(impl Foo)) {
    let foo = S;
    f(foo);
}

// This caused stack overflow in #17498
fn not_allowed2(f: impl Fn(&impl Foo)) {
    let foo = S;
    f(&foo);
}

fn not_allowed3(bar: impl Bar<impl Foo>) {}

// This also caused stack overflow
fn not_allowed4(bar: impl Bar<&impl Foo>) {}

fn allowed1(baz: impl Baz<Assoc = impl Foo>) {}

fn allowed2<'a>(baz: impl Baz<Assoc = &'a (impl Foo + 'a)>) {}

fn allowed3(baz: impl Baz<Assoc = Qux<impl Foo>>) {}
",
        expect![[r#"
            fn foo<'a, const C: usize = 314235, T = B>() -> &'a dyn Fn::<(), Output = i32>
            where
                T: Trait::<Item = A>,
                (): Default
             {...}
            fn impl_trait_args<T, Param[1]>()
            where
                Param[1]: Trait
             {...}
            fn impl_trait_args2<T, Param[1]>()
            where
                Param[1]: Trait::<{error}>
             {...}
            fn impl_trait_ret<T>() -> impl Trait {...}
            fn impl_trait_ret2<T>() -> impl Trait::<{error}> {...}
            fn not_allowed1<Param[0]>()
            where
                Param[0]: Fn::<({error}), Output = ()>
             {...}
            fn not_allowed2<Param[0]>()
            where
                Param[0]: Fn::<(&{error}), Output = ()>
             {...}
            fn not_allowed3<Param[0]>()
            where
                Param[0]: Bar::<{error}>
             {...}
            fn not_allowed4<Param[0]>()
            where
                Param[0]: Bar::<&{error}>
             {...}
            fn allowed1<Param[0], Param[1]>()
            where
                Param[0]: Foo,
                Param[1]: Baz::<Assoc = Param[0]>
             {...}
            fn allowed2<'a, Param[0], Param[1]>()
            where
                Param[0]: Foo,
                Param[0]: 'a,
                Param[1]: Baz::<Assoc = &'a Param[0]>
             {...}
            fn allowed3<Param[0], Param[1]>()
            where
                Param[0]: Foo,
                Param[1]: Baz::<Assoc = Qux::<Param[0]>>
             {...}
        "#]],
    );
}
