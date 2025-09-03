use hir::{EditionedFileId, Semantics};
use ide_db::{
    RootDatabase,
    defs::Definition,
    search::{FileReference, FileReferenceNode},
};
use syntax::{
    AstNode, T,
    ast::{self, edit::AstNodeEdit, make},
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: generate_inverse_ops_impl
//
// Generate inverse ops implement.
//
// ```
// struct Foo(i32);
// impl core::ops::Add<i32> $0for Foo {
//     type Output = i32;
//
//     fn add(self, rhs: i32) -> Self::Output {
//         self.0 + rhs
//     }
// }
// ```
// ->
// ```
// struct Foo(i32);
// impl core::ops::Add<Foo> for i32 {
//     type Output = i32;
//
//     fn add(self, rhs: Foo) -> Self::Output {
//         rhs.0 + self
//     }
// }
//
// impl core::ops::Add<i32> for Foo {
//     type Output = i32;
//
//     fn add(self, rhs: i32) -> Self::Output {
//         self.0 + rhs
//     }
// }
// ```
pub(crate) fn generate_inverse_ops_impl(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let for_kw = ctx.find_token_syntax_at_offset(T![for])?;
    let parent = for_kw.parent()?;
    let impl_ = ast::Impl::cast(parent)?;
    let trait_ = impl_.trait_()?;
    let self_ty = impl_.self_ty()?;

    let ast::GenericArg::TypeArg(gen_arg) = trait_.generic_arg_list()?.generic_args().next()?
    else {
        return None;
    };
    let ast::Type::PathType(trait_path) = trait_ else {
        return None;
    };
    let gen_ty = gen_arg.ty()?;
    if gen_ty.syntax().text() == "Self" || gen_ty.syntax().text() == self_ty.syntax().text() {
        return None;
    }

    let trait_name = trait_path.path()?.segment()?.name_ref()?;
    valided_ops_trait(&trait_name.text())?;

    let target = for_kw.text_range();
    acc.add(
        AssistId::generate("generate_inverse_ops_impl"),
        format!("Generate inverse `impl {trait_name}<{self_ty}> for {gen_arg}`"),
        target,
        |builder| {
            let mut edit = builder.make_editor(impl_.syntax());

            let new_impl = impl_.clone_subtree();
            let mut new_edit = SyntaxEditor::new(new_impl.syntax().clone());

            if let Some(params) = find_param_list(&new_impl) {
                params
                    .syntax()
                    .descendants()
                    .filter_map(ast::Type::cast)
                    .filter(|ty| ty.syntax().text() == gen_ty.syntax().text())
                    .for_each(|ty| {
                        new_edit.replace(ty.syntax(), self_ty.clone_for_update().syntax());
                    });
            }
            if let Some((new_trait, new_self)) = new_impl.trait_().zip(new_impl.self_ty())
                && let Some(new_gen) = new_trait.generic_arg_list()
                && let Some(new_gen_arg) = new_gen.generic_args().next()
            {
                new_edit.replace(new_gen_arg.syntax(), new_self.syntax());
                new_edit.replace(new_self.syntax(), new_gen_arg.syntax());
            }
            let _ = rename_params(&impl_, &new_impl, &ctx.sema, &mut new_edit, ctx.file_id());

            edit.insert_all(
                Position::before(impl_.syntax()),
                vec![
                    new_edit.finish().new_root().clone().into(),
                    make::tokens::whitespace(&format!("\n\n{}", impl_.indent_level())).into(),
                ],
            );

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn rename_params(
    impl_: &ast::Impl,
    new_impl: &ast::Impl,
    sema: &Semantics<'_, RootDatabase>,
    edit: &mut SyntaxEditor,
    file_id: EditionedFileId,
) -> Option<()> {
    let method = impl_.assoc_item_list()?.assoc_items().find_map(|it| match it {
        ast::AssocItem::Fn(f) => Some(f),
        _ => None,
    })?;
    let db = sema.db;
    let def = sema.to_def(&method)?;
    let mut params = def.assoc_fn_params(db);
    let rhs_param = params.pop()?;
    let self_param = params.pop()?;

    let start = impl_.syntax().text_range().start();
    let mut replace_usages = |param: hir::Local, new: &str| {
        Definition::Local(param)
            .usages(sema)
            .all()
            .references
            .remove(&file_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|FileReference { name, range, .. }| match name {
                FileReferenceNode::NameRef(_) => Some(range),
                _ => None,
            })
            .for_each(|range| {
                let elem = new_impl.syntax().covering_element(range - start);
                edit.replace(elem, make::name_ref(new).clone_for_update().syntax());
            })
    };
    replace_usages(self_param.as_local(db)?, rhs_param.name(db)?.as_str());
    replace_usages(rhs_param.as_local(db)?, self_param.name(db)?.as_str());

    Some(())
}

fn find_param_list(impl_: &ast::Impl) -> Option<ast::ParamList> {
    impl_
        .assoc_item_list()?
        .assoc_items()
        .find_map(|it| if let ast::AssocItem::Fn(f) = it { Some(f) } else { None })?
        .param_list()
}

fn valided_ops_trait(trait_name: &str) -> Option<()> {
    match trait_name {
        "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Shl" | "Shr" | "BitAnd" | "BitOr" | "BitXor"
        | "AddAssign" | "SubAssign" | "MulAssign" | "DivAssign" | "RemAssign" | "ShlAssign"
        | "ShrAssign" | "BitAndAssign" | "BitOrAssign" | "BitXorAssign" => Some(()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_generate_inverse_ops_impl() {
        check_assist(
            generate_inverse_ops_impl,
            "
struct Foo;
impl core::ops::Add<i32> $0for Foo {
    type Output = i32;

    fn add(self, rhs: i32) -> Self::Output {
        todo!()
    }
}
            ",
            "
struct Foo;
impl core::ops::Add<Foo> for i32 {
    type Output = i32;

    fn add(self, rhs: Foo) -> Self::Output {
        todo!()
    }
}

impl core::ops::Add<i32> for Foo {
    type Output = i32;

    fn add(self, rhs: i32) -> Self::Output {
        todo!()
    }
}
            ",
        );
    }

    #[test]
    fn test_generate_inverse_ops_impl_not_applicable_self() {
        check_assist_not_applicable(
            generate_inverse_ops_impl,
            "
struct Foo;
impl core::ops::Add<Foo> $0for Foo {
    type Output = i32;

    fn add(self, rhs: Foo) -> Self::Output {
        todo!()
    }
}
            ",
        );
    }
}
