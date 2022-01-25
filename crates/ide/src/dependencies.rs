#[cfg(test)]
mod tests {
    use expect_test::Expect;
    use expect_test::expect;
    use hir::Function;
    use hir::dependencies::function_to_checkable_code;
    use ide_db::RootDatabase;
    use ide_db::defs::IdentClass;
    use ide_db::helpers::pick_best_token;
    use itertools::Itertools;
    use std::iter;
    use syntax::AstNode;
    use syntax::SyntaxKind::*;
    use syntax::T;
    use crate::fixture;

    fn function_of_fixture(ra_fixture: &str, f: impl FnOnce(&RootDatabase, Function) + std::panic::UnwindSafe) {
        let (a, position) = fixture::position(ra_fixture);
        a.with_db(|db| {
            let sema = &hir::Semantics::new(db);
            let file = sema.parse(position.file_id).syntax().clone();

            let offset = position.offset;

            let original_token = pick_best_token(file.token_at_offset(offset), |kind| match kind {
                IDENT | INT_NUMBER | LIFETIME_IDENT | T![self] | T![super] | T![crate] => 3,
                T!['('] | T![')'] => 2,
                kind if kind.is_trivia() => 0,
                _ => 1,
            })
            .unwrap();

            let descended = sema.descend_into_macros(original_token.clone());

            let result = descended
                .iter()
                .filter_map(|token| {
                    let node = token.parent()?;
                    let class = IdentClass::classify_token(sema, token)?;
                    Some(class.definitions().into_iter().zip(iter::once(node).cycle()))
                })
                .flatten()
                .unique_by(|&(def, _)| def)
                .find_map(|(def, _)| match def {
                    ide_db::defs::Definition::Function(f) => Some(f),
                    _ => None,
                })
                .unwrap();
            f(db, result);
            Some(())
        }).unwrap();
    }

    fn check_text(ra_fixture: &str, code: Expect) {
        function_of_fixture(ra_fixture, |db, func| {
            code.assert_eq(&function_to_checkable_code(db, func).unwrap());
        })
    }

    #[test]
    fn simple() {
        check_text(
            r#"
        //- minicore: option, clone, derive
        enum Gav {
            Var1(Option<Baz>),
            Var2,
        }
        struct Baz(i32);
        #[derive(Clone)]
        struct Bar;
        #[derive(Clone)]
        struct Foo(i32, Bar, Bar);

        fn bar(x: Gav) -> i32 {
            match x {
                Var1 => Baz(34).0,
                Var2 => 43,
            }
        }

        fn foo_to_gav(x: Foo) -> Gav {
            todo!()
        }

        fn fo$0o(x: core::option::Option<Foo>) -> i32 {
            if let Some(x) = x {
                bar(foo_to_gav(x.clone()))
            } else {
                5
            }
        }
        "#,
            expect![[r"struct _ra_S1;struct _ra_S2(i32,_ra_S1,_ra_S1,);pub enum _ra_E0<T>{None,Some(T,),}struct _ra_S0(i32,);enum _ra_E1{Var1(_ra_E0<_ra_S0>,),Var2,}fn _ra_f0(x: _ra_E1) -> i32{loop{}}fn _ra_f1(x: _ra_S2) -> _ra_E1{loop{}}pub trait _ra_T1
where
    Self: Sized,{pub fn clone(&self) -> Self{loop{}}}impl _ra_T1 for _ra_S2 {}
pub(crate) use _ra_S2 as Foo;pub(crate) use _ra_f1 as foo_to_gav;pub(crate) use _ra_E0::Some as Some;pub(crate) use _ra_f0 as bar;pub mod core {pub mod option {pub(crate) use super::super::_ra_E0 as Option;}}
fn main() { let _ = foo; }
fn foo(x: core::option::Option<Foo>) -> i32 {
    if let Some(x) = x {
        bar(foo_to_gav(x.clone()))
    } else {
        5
    }
}"]],
        );
    }
}

