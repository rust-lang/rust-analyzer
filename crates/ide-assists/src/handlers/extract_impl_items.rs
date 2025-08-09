use crate::assist_context::{AssistContext, Assists};
use ide_db::assists::AssistId;
use syntax::{
    AstNode, SyntaxKind, TextRange,
    ast::{self, edit_in_place::Indent, make},
    syntax_editor::Position,
    ted,
};

// Assist: extract_impl_items
//
// Extract selected impl items into new impl.
//
// ```
// struct Foo;
// impl Foo {
//     fn foo() {}
//     $0fn bar() {}
//     fn baz() {}$0
// }
// ```
// ->
// ```
// struct Foo;
// impl Foo {
//     fn foo() {}
// }
//
// impl Foo {
//     fn bar() {}
//
//     fn baz() {}
// }
// ```
pub(crate) fn extract_impl_items(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    if ctx.has_empty_selection() {
        return None;
    }
    let item = ctx.find_node_at_trimmed_offset::<ast::AssocItem>()?;
    let impl_ = ast::Impl::cast(item.syntax().parent()?.parent()?)?;
    let indent = impl_.indent_level();
    let selection = ctx.selection_trimmed();
    let items = selection_items(&impl_, selection)?;

    let target = TextRange::new(
        items.first()?.syntax().text_range().start(),
        items.last()?.syntax().text_range().end(),
    );
    acc.add(
        AssistId::refactor_extract("extract_impl_items"),
        "Extract items into new impl block",
        target,
        |builder| {
            let mut edit = builder.make_editor(impl_.syntax());
            let new_impl = impl_.clone_for_update();

            for origin_item in &items {
                if let Some(token) = origin_item.syntax().prev_sibling_or_token()
                    && token.kind() == SyntaxKind::WHITESPACE
                {
                    edit.delete(token);
                }
                edit.delete(origin_item.syntax());
            }

            if let Some(assoc_item_list) = new_impl.assoc_item_list() {
                let new_item_list = make::assoc_item_list(None);
                ted::replace(assoc_item_list.syntax(), new_item_list.clone_for_update().syntax());
            }

            let assoc_item_list = new_impl.get_or_create_assoc_item_list();

            for item in items {
                assoc_item_list.add_item(item.clone_for_update());
            }

            edit.insert_all(
                Position::after(impl_.syntax()),
                vec![
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                    new_impl.syntax().clone().into(),
                ],
            );

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn selection_items(impl_: &ast::Impl, selection: TextRange) -> Option<Vec<ast::AssocItem>> {
    let items = impl_
        .assoc_item_list()?
        .assoc_items()
        .filter(|item| {
            let item_range = item.syntax().text_range();
            selection == item_range
                || selection.intersect(item_range).is_some_and(|range| !range.is_empty())
                    && !item_range.contains_range(selection)
        })
        .collect::<Vec<_>>();
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn test_extract_impl_items() {
        check_assist(
            extract_impl_items,
            r#"
struct Foo;
impl Foo {
    fn other_1() {}

    /// some docs

    /// some docs
    fn $0extract_1() {}

    // some comment

    fn extract_2() {}

    const EXTRACT_3: u32 = 2;$0

    fn other_2() {}
}
            "#,
            r#"
struct Foo;
impl Foo {
    fn other_1() {}

    // some comment

    fn other_2() {}
}

impl Foo {
    /// some docs

    /// some docs
    fn extract_1() {}

    fn extract_2() {}

    const EXTRACT_3: u32 = 2;
}
            "#,
        );
    }

    #[test]
    fn test_extract_impl_items_with_generics() {
        check_assist(
            extract_impl_items,
            r#"
struct Foo<T>(T);
impl<T> Foo<T>
where
    T: Clone,
{
    fn other_1() {}

    /// some docs
    fn $0extract_1() {}

    // some comment
    fn extract_2() {}

    const EXTRACT_3: u32 = 2;$0

    fn other_2() {}
}
            "#,
            r#"
struct Foo<T>(T);
impl<T> Foo<T>
where
    T: Clone,
{
    fn other_1() {}

    fn other_2() {}
}

impl<T> Foo<T>
where
    T: Clone,
{
    /// some docs
    fn extract_1() {}

    // some comment
    fn extract_2() {}

    const EXTRACT_3: u32 = 2;
}
            "#,
        );
    }

    #[test]
    fn test_extract_impl_items_with_indent() {
        check_assist(
            extract_impl_items,
            r#"
mod foo {
    mod bar {
        struct Foo;
        impl Foo {
            fn other_1() {
                todo!()
            }

            $0fn extract_1() {
                todo!()
            }$0
        }
    }
}
            "#,
            r#"
mod foo {
    mod bar {
        struct Foo;
        impl Foo {
            fn other_1() {
                todo!()
            }
        }

        impl Foo {
            fn extract_1() {
                todo!()
            }
        }
    }
}
            "#,
        );
    }
    #[test]
    fn test_extract_impl_items_in_body_not_application() {
        check_assist_not_applicable(
            extract_impl_items,
            r#"
struct Foo;
impl Foo {
    fn other_1() {
        todo!()
    }

    fn other_1() {
        $0()$0
    }
}
            "#,
        );
    }
}
