use ast::Name;
use either::Either::{self, Left, Right};
use ide_db::{source_change::SourceChangeBuilder, syntax_helpers::suggest_name::NameGenerator};
use syntax::{
    ast::{self, AstNode, HasGenericParams, HasName, make},
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: extract_to_default_generic
//
// Extracts selected type to default generic parameter.
//
// ```
// struct Foo(u32, $0String$0);
// ```
// ->
// ```
// struct Foo<T$0 = String>(u32, T);
// ```
pub(crate) fn extract_to_default_generic(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    if ctx.has_empty_selection() {
        return None;
    }

    let ty: Either<ast::Type, ast::ConstArg> = ctx.find_node_at_range()?;
    let adt: Either<Either<ast::Adt, ast::TypeAlias>, ast::Fn> =
        ty.syntax().ancestors().find_map(AstNode::cast)?;

    extract_to_default_generic_impl(acc, ctx, adt, ty)
}

fn extract_to_default_generic_impl(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
    adt: Either<impl HasName + HasGenericParams, ast::Fn>,
    ty: Either<ast::Type, ast::ConstArg>,
) -> Option<()> {
    let name = adt.name()?;
    let has_default = adt.is_left();

    let label = if has_default {
        "Extract type as default generic parameter"
    } else {
        "Extract type as generic parameter"
    };
    let target = ty.syntax().text_range();
    acc.add(AssistId::refactor_extract("extract_to_default_generic"), label, target, |edit| {
        let mut editor = edit.make_editor(adt.syntax());
        let generic_list = get_or_create_generic_param_list(&name, &adt, &mut editor, edit);

        let generic_name = generic_name(&generic_list, ty.is_right());

        editor.replace(ty.syntax(), generic_name.syntax());

        match ty {
            Left(ty) => {
                let param = if has_default {
                    make::type_default_param(generic_name, None, ty)
                } else {
                    make::type_param(generic_name, None)
                }
                .clone_for_update();

                generic_list.add_generic_param(param.into());
            }
            Right(n) => {
                let param = if has_default {
                    make::const_default_param(generic_name, const_ty(ctx, &n), n)
                } else {
                    make::const_param(generic_name, const_ty(ctx, &n))
                }
                .clone_for_update();

                generic_list.add_generic_param(param.into());
            }
        }

        if let Some(cap) = ctx.config.snippet_cap
            && let Some(last) = generic_list.generic_params().last()
        {
            if let ast::GenericParam::TypeParam(param) = &last
                && let Some(name) = param.name()
            {
                let annotation = edit.make_tabstop_after(cap);
                editor.add_annotation(name.syntax(), annotation);
            } else if let ast::GenericParam::ConstParam(param) = &last
                && let Some(ast::Type::InferType(ty)) = param.ty()
            {
                let annotation = edit.make_placeholder_snippet(cap);
                editor.add_annotation(ty.syntax(), annotation);
            }
        }

        edit.add_file_edits(ctx.vfs_file_id(), editor);
    })
}

fn array_index_type(n: &ast::ConstArg) -> Option<ast::Type> {
    let kind = n.syntax().parent()?.kind();

    if ast::ArrayType::can_cast(kind) || ast::ArrayExpr::can_cast(kind) {
        Some(make::ty("usize"))
    } else {
        None
    }
}

fn generic_name(generic_list: &ast::GenericParamList, is_const_param: bool) -> Name {
    let exist_names = generic_list
        .generic_params()
        .filter_map(|it| match it {
            ast::GenericParam::ConstParam(const_param) => const_param.name(),
            ast::GenericParam::TypeParam(type_param) => type_param.name(),
            ast::GenericParam::LifetimeParam(_) => None,
        })
        .map(|name| name.to_string())
        .collect::<Vec<_>>();

    let mut name_gen = NameGenerator::new_with_names(exist_names.iter().map(|name| name.as_str()));

    make::name(&if is_const_param {
        name_gen.suggest_name("N")
    } else {
        name_gen.suggest_name("T")
    })
    .clone_for_update()
}

fn const_ty(ctx: &AssistContext<'_>, n: &ast::ConstArg) -> ast::Type {
    if let Some(expr) = n.expr()
        && let Some(ty_info) = ctx.sema.type_of_expr(&expr)
        && let Some(builtin) = ty_info.adjusted().as_builtin()
    {
        make::ty(builtin.name().as_str())
    } else if let Some(array_index_ty) = array_index_type(n) {
        array_index_ty
    } else {
        make::ty_placeholder()
    }
}

fn get_or_create_generic_param_list(
    name: &ast::Name,
    adt: &impl HasGenericParams,
    editor: &mut SyntaxEditor,
    edit: &mut SourceChangeBuilder,
) -> ast::GenericParamList {
    if let Some(list) = adt.generic_param_list() {
        edit.make_mut(list)
    } else {
        let generic = make::generic_param_list([]).clone_for_update();
        editor.insert(Position::after(name.syntax()), generic.syntax());
        generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_extract_to_default_generic() {
        check_assist(
            extract_to_default_generic,
            r#"type X = ($0i32$0, i64);"#,
            r#"type X<T$0 = i32> = (T, i64);"#,
        );

        check_assist(
            extract_to_default_generic,
            r#"type X<T> = ($0i32$0, T);"#,
            r#"type X<T, T1$0 = i32> = (T1, T);"#,
        );
    }

    #[test]
    fn test_extract_to_default_generic_on_adt() {
        check_assist(
            extract_to_default_generic,
            r#"struct Foo($0i32$0);"#,
            r#"struct Foo<T$0 = i32>(T);"#,
        );

        check_assist(
            extract_to_default_generic,
            r#"struct Foo<T>(T, $0i32$0);"#,
            r#"struct Foo<T, T1$0 = i32>(T, T1);"#,
        );

        check_assist(
            extract_to_default_generic,
            r#"enum Foo { A($0i32$0), B, C(i64) };"#,
            r#"enum Foo<T$0 = i32> { A(T), B, C(i64) };"#,
        );
    }

    #[test]
    fn test_extract_to_generic_on_fn() {
        check_assist(
            extract_to_default_generic,
            r#"fn foo(x: $0i32$0) {}"#,
            r#"fn foo<T$0>(x: T) {}"#,
        );

        check_assist(
            extract_to_default_generic,
            r#"fn foo(x: [i32; $02$0]) {}"#,
            r#"fn foo<const N: usize>(x: [i32; N]) {}"#,
        );
    }

    #[test]
    fn test_extract_to_default_generic_const() {
        check_assist(
            extract_to_default_generic,
            r#"type A = [i32; $08$0];"#,
            r#"type A<const N: usize = 8> = [i32; N];"#,
        );

        check_assist(
            extract_to_default_generic,
            r#"type A<T> = [T; $08$0];"#,
            r#"type A<T, const N: usize = 8> = [T; N];"#,
        );
    }

    #[test]
    fn test_extract_to_default_generic_const_non_array() {
        check_assist(
            extract_to_default_generic,
            r#"
                struct Foo<const N: usize>([(); N]);
                type A = Foo<$08$0>;
            "#,
            r#"
                struct Foo<const N: usize>([(); N]);
                type A<const N: ${0:_} = 8> = Foo<N>;
            "#,
        );
    }
}
