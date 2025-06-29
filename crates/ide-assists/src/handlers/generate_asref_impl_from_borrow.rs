use syntax::{
    ast::{self, AstNode, HasName, edit_in_place::Indent, make},
    syntax_editor::Position,
};

use crate::{AssistContext, AssistId, Assists};

// Assist: generate_asref_impl_from_borrow
//
// Generate `AsRef` implement from `Borrow`.
//
// ```
// struct Foo<T>(T);
//
// impl<T> $0Borrow<T> for Foo<T> {
//     fn borrow(&self) -> &T {
//         &self.0
//     }
// }
// ```
// ->
// ```
// struct Foo<T>(T);
//
// $0impl<T> AsRef<T> for Foo<T> {
//     fn as_ref(&self) -> &T {
//         &self.0
//     }
// }
//
// impl<T> Borrow<T> for Foo<T> {
//     fn borrow(&self) -> &T {
//         &self.0
//     }
// }
// ```
//
// ---
// Generate `AsMut` implement from `BorrowMut`.
//
// ```
// struct Foo<T>(T);
//
// impl<T> $0BorrowMut<T> for Foo<T> {
//     fn borrow_mut(&mut self) -> &mut T {
//         &mut self.0
//     }
// }
// ```
// ->
// ```
// struct Foo<T>(T);
//
// $0impl<T> AsMut<T> for Foo<T> {
//     fn as_mut(&mut self) -> &mut T {
//         &mut self.0
//     }
// }
//
// impl<T> BorrowMut<T> for Foo<T> {
//     fn borrow_mut(&mut self) -> &mut T {
//         &mut self.0
//     }
// }
// ```
pub(crate) fn generate_asref_impl_from_borrow(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let ty = ctx.find_node_at_offset::<ast::Type>()?;
    let impl_ = ast::Impl::cast(ty.syntax().parent()?)?;
    let path = ast::PathType::cast(impl_.trait_()?.syntax().clone())?;
    let indent = impl_.indent_level();

    let name = path.path()?.segment()?.name_ref()?;

    let (target_name, target_method_name) = match &*name.text() {
        "Borrow" => ("AsRef", "as_ref"),
        "BorrowMut" => ("AsMut", "as_mut"),
        _ => return None,
    };

    let method = impl_.assoc_item_list()?.assoc_items().find_map(|it| match it {
        ast::AssocItem::Fn(f) => Some(f),
        _ => None,
    })?;

    let target = impl_.syntax().text_range();
    acc.add(
        AssistId::generate("generate_asref_impl_from_borrow"),
        format!("Generate `{target_name}` implement from `{name}`"),
        target,
        |edit| {
            let mut editor = edit.make_editor(impl_.syntax());
            editor.replace(name.syntax(), make::name_ref(target_name).syntax().clone_for_update());

            if let Some(name) = method.name() {
                editor.replace(
                    name.syntax(),
                    make::name(target_method_name).syntax().clone_for_update(),
                );
            }

            editor.insert_all(
                Position::after(impl_.syntax()),
                vec![
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                    impl_.syntax().clone_for_update().into(),
                ],
            );

            if let Some(cap) = ctx.config.snippet_cap {
                edit.add_tabstop_before(cap, impl_);
            }

            edit.add_file_edits(ctx.vfs_file_id(), editor);
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn test_generate_asref_impl_from_borrow() {
        check_assist(
            generate_asref_impl_from_borrow,
            r#"
struct Foo<T>(T);

impl<T> $0Borrow<T> for Foo<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}
            "#,
            r#"
struct Foo<T>(T);

$0impl<T> AsRef<T> for Foo<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for Foo<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}
            "#,
        );
    }

    #[test]
    fn test_generate_asmut_impl_from_borrow_mut() {
        check_assist(
            generate_asref_impl_from_borrow,
            r#"
struct Foo<T>(T);

impl<T> $0BorrowMut<T> for Foo<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
            "#,
            r#"
struct Foo<T>(T);

$0impl<T> AsMut<T> for Foo<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> BorrowMut<T> for Foo<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
            "#,
        );
    }

    #[test]
    fn test_generate_asref_impl_from_borrow_attributes() {
        check_assist(
            generate_asref_impl_from_borrow,
            r#"
struct Foo<T>(T);

#[cfg(feature = "foo")]
impl<T> $0Borrow<T> for Foo<T> {
    /// some docs
    fn borrow(&self) -> &T {
        &self.0
    }
}
            "#,
            r#"
struct Foo<T>(T);

$0#[cfg(feature = "foo")]
impl<T> AsRef<T> for Foo<T> {
    /// some docs
    fn as_ref(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "foo")]
impl<T> Borrow<T> for Foo<T> {
    /// some docs
    fn borrow(&self) -> &T {
        &self.0
    }
}
            "#,
        );
    }

    #[test]
    fn test_generate_asref_impl_from_borrow_indent() {
        check_assist(
            generate_asref_impl_from_borrow,
            r#"
mod foo {
    mod bar {
        struct Foo<T>(T);

        impl<T> $0Borrow<T> for Foo<T> {
            fn borrow(&self) -> &T {
                &self.0
            }
        }
    }
}
            "#,
            r#"
mod foo {
    mod bar {
        struct Foo<T>(T);

        $0impl<T> AsRef<T> for Foo<T> {
            fn as_ref(&self) -> &T {
                &self.0
            }
        }

        impl<T> Borrow<T> for Foo<T> {
            fn borrow(&self) -> &T {
                &self.0
            }
        }
    }
}
            "#,
        );
    }
}
