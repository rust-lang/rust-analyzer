use std::iter::empty;

use either::Either::{self, Left, Right};
use ide_db::{RootDatabase, syntax_helpers::suggest_name::NameGenerator};
use syntax::{
    AstNode, SmolStr, SyntaxKind, T,
    ast::{
        self, HasGenericParams, HasName, HasTypeBounds, SelfParamKind, edit_in_place::Indent, make,
    },
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: generate_ref_impl
//
// Add a trait impl for reference.
//
// ```
// trait $0Foo {
//     fn foo(&mut self, n: i32) -> i32;
// }
// ```
// ->
// ```
// trait Foo {
//     fn foo(&mut self, n: i32) -> i32;
// }
//
// impl<T: Foo + ?Sized> Foo for &mut T {
//     fn foo(&mut self, n: i32) -> i32 {
//         (**self).foo(n)
//     }
// }
// ```
pub(crate) fn generate_ref_impl(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let name: ast::Name = ctx.find_node_at_offset()?;
    let trait_ = ast::Trait::cast(name.syntax().parent()?)?;

    let _ = generate(acc, ctx, &trait_, SelfParamKind::Ref);
    let _ = generate(acc, ctx, &trait_, SelfParamKind::MutRef);
    Some(())
}

fn generate(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
    trait_: &ast::Trait,
    out: SelfParamKind,
) -> Option<()> {
    let name = trait_.name()?;
    let items = trait_.assoc_item_list()?;
    let exclusive = matches!(out, SelfParamKind::MutRef);
    let self_ty = make::ty_ref(make::ty("T"), exclusive);
    let indent = trait_.indent_level();

    check_methods(out, &items)?;
    if check_existing(out, &ctx.sema, trait_).is_some() {
        return None;
    }

    let target = trait_.syntax().text_range();
    acc.add(
        AssistId::generate("generate_ref_impl"),
        format!("Generate `impl<T: {name} + ?Sized> {name} for {self_ty}`"),
        target,
        |builder| {
            let trait_gen_args = trait_.generic_param_list().map(|p| p.to_generic_args());

            let param_name = &generic_param_name(trait_);
            let self_ty = make::ty_ref(make::ty(param_name), exclusive);
            let impl_ = make::impl_trait(
                None,
                trait_.unsafe_token().is_some(),
                trait_.generic_param_list(),
                trait_gen_args,
                Some(make::generic_param_list([make::type_param(
                    make::name(param_name),
                    make::type_bound_list([
                        make::type_bound_text(&name.text()),
                        make::type_bound_text("?Sized"),
                    ]),
                )
                .into()])),
                None,
                false,
                make::ty(&name.text()),
                self_ty,
                trait_.where_clause(),
                None,
                Some(items.clone_for_update()),
            )
            .clone_subtree();

            let mut edit = SyntaxEditor::new(impl_.syntax().clone());

            for item in impl_.get_or_create_assoc_item_list().assoc_items() {
                process_assoc_item(&mut edit, &item, param_name);
            }

            let impl_ = edit.finish().new_root().clone();
            let mut edit = builder.make_editor(trait_.syntax());

            edit.insert_all(
                Position::after(trait_.syntax()),
                vec![make::tokens::whitespace(&format!("\n\n{indent}")).into(), impl_.into()],
            );

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn check_existing(
    out: SelfParamKind,
    sema: &hir::Semantics<'_, RootDatabase>,
    trait_: &ast::Trait,
) -> Option<()> {
    let def = sema.to_def(trait_)?;
    let module = def.module(sema.db);
    let impls = module.impl_defs(sema.db);
    let predicate = |ty: hir::Type<'_>| match out {
        SelfParamKind::Owned => false,
        SelfParamKind::Ref => ty.is_reference(),
        SelfParamKind::MutRef => ty.is_mutable_reference(),
    };
    impls
        .iter()
        .any(|impl_| {
            impl_.trait_(sema.db).as_ref() == Some(&def) && predicate(impl_.self_ty(sema.db))
        })
        .then_some(())
}

fn generic_param_name(trait_: &ast::Trait) -> SmolStr {
    let mut names_gen = NameGenerator::new_with_names(empty());

    trait_
        .generic_param_list()
        .iter()
        .flat_map(|params| params.generic_params())
        .map(|it| it.syntax().clone())
        .chain(trait_.assoc_item_list().into_iter().flat_map(|items| items.assoc_items()).flat_map(
            |item| {
                ast::AnyHasGenericParams::cast(item.syntax().clone())
                    .and_then(|it| it.generic_param_list())
                    .into_iter()
                    .flat_map(|it| it.generic_params().map(|param| param.syntax().clone()))
                    .chain([item.syntax().clone()])
            },
        ))
        .filter_map(ast::AnyHasName::cast)
        .filter_map(|it| it.name())
        .for_each(|name| _ = names_gen.suggest_name(&name.text()));

    names_gen.suggest_name("T")
}

fn process_assoc_item(edit: &mut SyntaxEditor, origin: &ast::AssocItem, param_name: &str) {
    let Some(item) =
        Either::<ast::Fn, Either<ast::TypeAlias, ast::Const>>::cast(origin.syntax().clone())
    else {
        return;
    };
    match &item {
        Left(f) => process_method(edit, f),
        Right(right) => {
            let Some(name) = item.name().map(|name| make::name_ref(&name.text())) else {
                return;
            };
            let path = assoc_path(param_name, name, &item);
            match right {
                Left(type_item) => process_type_alias(edit, path, type_item),
                Right(const_item) => process_const_item(edit, path, const_item),
            }
        }
    }
}

fn assoc_path(param_name: &str, assoc_name: ast::NameRef, node: &impl AstNode) -> ast::Path {
    let top_segment = if let Some(item) = ast::AnyHasGenericParams::cast(node.syntax().clone())
        && let Some(params) = item.generic_param_list()
    {
        make::generic_ty_path_segment(assoc_name, params.to_generic_args().generic_args())
    } else {
        make::path_segment(assoc_name)
    };
    make::path_from_segments([make::path_segment(make::name_ref(param_name)), top_segment], false)
        .clone_for_update()
}

fn process_method(edit: &mut SyntaxEditor, f: &ast::Fn) {
    let method = f
        .name()
        .map(|name| make::name_ref(&name.text()))
        .unwrap_or_else(|| make::name_ref("unnamed"));
    let self_expr = make::expr_name_ref(make::name_ref("self"));
    let inner = make::expr_paren(
        make::expr_prefix(T![*], make::expr_prefix(T![*], self_expr).into()).into(),
    )
    .into();

    let arg_list = generate_arg_list(edit, f.param_list());
    let call = make::expr_method_call(inner, method, arg_list);

    let block_expr = make::block_expr([], Some(call.into())).clone_for_update();
    block_expr.indent(f.indent_level());
    f.replace_or_insert_body(edit, block_expr);
}

fn process_type_alias(edit: &mut SyntaxEditor, path: ast::Path, type_item: &ast::TypeAlias) {
    if let Some(ty_bound) = type_item.type_bound_list() {
        edit.delete(ty_bound.syntax());
    }
    if let Some(colon) = type_item.colon_token() {
        if let Some(next_token) = colon.next_token()
            && next_token.kind() == SyntaxKind::WHITESPACE
        {
            edit.delete(next_token);
        }
        edit.delete(colon);
    }
    if type_item.eq_token().is_none()
        && let Some(token) = type_item
            .generic_param_list()
            .map(|it| it.syntax().clone())
            .or_else(|| type_item.name().map(|it| it.syntax().clone()))
            .and_then(|it| it.last_token())
    {
        edit.insert_all(
            Position::after(&token),
            vec![
                make::tokens::single_space().into(),
                make::token(T![=]).into(),
                make::tokens::single_space().into(),
                path.syntax().clone().into(),
            ],
        );
    } else if let Some(ty) = type_item.ty() {
        edit.replace(ty.syntax(), path.syntax());
    }
}

fn process_const_item(edit: &mut SyntaxEditor, path: ast::Path, const_item: &ast::Const) {
    if let Some(ty) = const_item.ty()
        && const_item.eq_token().is_none()
    {
        edit.insert_all(
            Position::after(ty.syntax()),
            vec![
                make::tokens::single_space().into(),
                make::token(T![=]).into(),
                make::tokens::single_space().into(),
                path.syntax().clone().into(),
            ],
        );
    } else if let Some(init) = const_item.body() {
        edit.replace(init.syntax(), path.syntax());
    }
}

fn generate_arg_list(edit: &mut SyntaxEditor, params: Option<ast::ParamList>) -> ast::ArgList {
    let Some(params) = params else {
        return make::arg_list([]);
    };
    let mut names_gen = NameGenerator::new_with_names(empty());
    let args = params
        .params()
        .map(|x| {
            (
                x.pat(),
                match x.pat() {
                    Some(ast::Pat::IdentPat(name)) => name.name(),
                    _ => None,
                },
            )
        })
        .map(|it| match it {
            (_, Some(name)) => make::expr_name_ref(make::name_ref(&name.text())),
            (Some(pat), None) => {
                let name = names_gen.suggest_name("arg");
                let arg = make::expr_name_ref(make::name_ref(&name));
                edit.replace(pat.syntax(), arg.clone_for_update().syntax());
                arg
            }
            (None, None) => make::ext::expr_underscore(),
        });
    make::arg_list(args)
}

fn check_methods(out: SelfParamKind, items: &ast::AssocItemList) -> Option<()> {
    for item in items.assoc_items() {
        let ast::AssocItem::Fn(f) = item else { continue };
        match f.param_list()?.self_param().map(|it| it.kind()) {
            Some(SelfParamKind::Owned) => return None,
            Some(SelfParamKind::MutRef) if out != SelfParamKind::MutRef => return None,
            _ => {}
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_generate_ref_impl() {
        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    fn foo(&mut self, n: i32) -> i32;
                }
            "#,
            r#"
                trait Foo {
                    fn foo(&mut self, n: i32) -> i32;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    fn foo(&mut self, n: i32) -> i32 {
                        (**self).foo(n)
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    fn foo(&mut self, n: i32) -> i32;

                    fn bar(&self, n: i32) -> i32;
                }
            "#,
            r#"
                trait Foo {
                    fn foo(&mut self, n: i32) -> i32;

                    fn bar(&self, n: i32) -> i32;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    fn foo(&mut self, n: i32) -> i32 {
                        (**self).foo(n)
                    }

                    fn bar(&self, n: i32) -> i32 {
                        (**self).bar(n)
                    }
                }
            "#,
        );
    }

    #[test]
    fn test_generate_ref_impl_existing() {
        check_assist_not_applicable(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    fn foo(&mut self, n: i32) -> i32;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    fn foo(&mut self, n: i32) -> i32 {
                        todo!()
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    fn foo(&self, n: i32) -> i32;
                }

                impl<T: Foo + ?Sized> Foo for &T {
                    fn foo(&self, n: i32) -> i32 {
                        (**self).foo(n)
                    }
                }
            "#,
            r#"
                trait Foo {
                    fn foo(&self, n: i32) -> i32;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    fn foo(&self, n: i32) -> i32 {
                        (**self).foo(n)
                    }
                }

                impl<T: Foo + ?Sized> Foo for &T {
                    fn foo(&self, n: i32) -> i32 {
                        (**self).foo(n)
                    }
                }
            "#,
        );
    }

    #[test]
    fn test_generate_ref_impl_immutable() {
        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    fn foo(&self, n: i32) -> i32;
                }
            "#,
            r#"
                trait Foo {
                    fn foo(&self, n: i32) -> i32;
                }

                impl<T: Foo + ?Sized> Foo for &T {
                    fn foo(&self, n: i32) -> i32 {
                        (**self).foo(n)
                    }
                }
            "#,
        );
    }

    #[test]
    fn test_generate_ref_impl_assoc_type() {
        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    type Item;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    type Item;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    type Item = T::Item;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    type Item: Copy;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    type Item: Copy;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    type Item = T::Item;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    type Item where Self: Sized;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    type Item where Self: Sized;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    type Item = T::Item where Self: Sized;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    type Item<T> where T: Sized;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    type Item<T> where T: Sized;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T1: Foo + ?Sized> Foo for &mut T1 {
                    type Item<T> = T1::Item<T> where T: Sized;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    type Item<T>;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    type Item<T>;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T1: Foo + ?Sized> Foo for &mut T1 {
                    type Item<T> = T1::Item<T>;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );
    }

    #[test]
    fn test_generate_ref_impl_assoc_const() {
        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    const N: usize;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    const N: usize;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T: Foo + ?Sized> Foo for &mut T {
                    const N: usize = T::N;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );

        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo {
                    const N<T>: usize;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }
            "#,
            r#"
                trait Foo {
                    const N<T>: usize;

                    fn foo(&mut self, n: i32) -> Self::Item;
                }

                impl<T1: Foo + ?Sized> Foo for &mut T1 {
                    const N<T>: usize = T1::N<T>;

                    fn foo(&mut self, n: i32) -> Self::Item {
                        (**self).foo(n)
                    }
                }
            "#,
        );
    }

    #[test]
    fn test_generate_ref_impl_rename() {
        check_assist(
            generate_ref_impl,
            r#"
                trait $0Foo<T> {
                    fn foo(&mut self, _: i32, _: T) -> i32;
                }
            "#,
            r#"
                trait Foo<T> {
                    fn foo(&mut self, _: i32, _: T) -> i32;
                }

                impl<T, T1: Foo + ?Sized> Foo<T> for &mut T1 {
                    fn foo(&mut self, arg: i32, arg1: T) -> i32 {
                        (**self).foo(arg, arg1)
                    }
                }
            "#,
        );
    }

    #[test]
    fn test_generate_ref_impl_with_indent() {
        check_assist(
            generate_ref_impl,
            r#"
                mod foo {
                    mod bar {
                        trait $0Foo {
                            fn foo(&mut self, n: i32) -> i32;
                        }
                    }
                }
            "#,
            r#"
                mod foo {
                    mod bar {
                        trait Foo {
                            fn foo(&mut self, n: i32) -> i32;
                        }

                        impl<T: Foo + ?Sized> Foo for &mut T {
                            fn foo(&mut self, n: i32) -> i32 {
                                (**self).foo(n)
                            }
                        }
                    }
                }
            "#,
        );
    }
}
