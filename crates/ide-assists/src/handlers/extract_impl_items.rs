use crate::assist_context::{AssistContext, Assists};
use ide_db::assists::AssistId;
use syntax::{
    AstNode, Direction, SyntaxElement, SyntaxKind, SyntaxToken, TextRange,
    ast::{self, edit_in_place::Indent, make},
    syntax_editor::{Element, Position, SyntaxEditor},
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
    let first = items.first()?;
    let last = items.last()?;
    let skip_items = impl_.assoc_item_list()?.assoc_items().position(|it| it == *first)?;

    let target =
        TextRange::new(first.syntax().text_range().start(), last.syntax().text_range().end());
    acc.add(
        AssistId::refactor_extract("extract_impl_items"),
        "Extract items into new impl block",
        target,
        |builder| {
            let mut edit = builder.make_editor(impl_.syntax());
            let new_impl = impl_.clone_subtree();
            let mut tedit = SyntaxEditor::new(new_impl.syntax().clone());

            if let Some(prev) = first.syntax().prev_sibling_or_token()
                && prev.kind() == SyntaxKind::WHITESPACE
            {
                edit.delete(prev);
            }
            edit.delete_all(first.syntax().syntax_element()..=last.syntax().syntax_element());
            let _ = exclude_delete(&mut tedit, &new_impl, skip_items, items.len());

            edit.insert_all(
                Position::after(impl_.syntax()),
                vec![
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                    tedit.finish().new_root().clone().into(),
                ],
            );

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn exclude_delete(
    tedit: &mut SyntaxEditor,
    new_impl: &ast::Impl,
    skip_items: usize,
    len: usize,
) -> Option<()> {
    let assoc_item_list = new_impl.assoc_item_list()?;
    let mut extracted = assoc_item_list.assoc_items().skip(skip_items).take(len);
    let first = extracted.next()?;
    let last = extracted.last().unwrap_or(first.clone());

    let l_curly = assoc_item_list.l_curly_token()?;
    let r_curly = assoc_item_list.r_curly_token()?;

    if let Some(start) = next_skip_ws(l_curly, Direction::Next)
        && let Some(end) = first.syntax().prev_sibling_or_token()
    {
        tedit.delete_all(start..=end);
    }

    if let Some(end) = next_skip_ws(r_curly, Direction::Prev)
        && let Some(start) = last.syntax().next_sibling_or_token()
    {
        tedit.delete_all(start..=end);
    }

    Some(())
}

fn next_skip_ws(token: SyntaxToken, dir: Direction) -> Option<SyntaxElement> {
    token.siblings_with_tokens(dir).skip(1).find(|it| it.kind() != SyntaxKind::WHITESPACE)
}

fn selection_items(impl_: &ast::Impl, selection: TextRange) -> Option<Vec<ast::AssocItem>> {
    let items = impl_
        .assoc_item_list()?
        .assoc_items()
        .skip_while(|item| item.syntax().text_range().end() < selection.start())
        .take_while(|item| item.syntax().text_range().start() <= selection.end())
        .collect::<Vec<_>>();

    let any_item_range = items.first()?.syntax().text_range();
    if any_item_range.contains_range(selection) && any_item_range != selection {
        return None;
    }
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

    // some comment

    /// some docs

    /// some docs
    fn $0extract_1() {}

    // some comment2

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

    // some comment2

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
