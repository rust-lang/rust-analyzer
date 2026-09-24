use syntax::{
    AstNode,
    SyntaxKind::{COMMENT, NEWLINE},
    TextRange,
    ast::{self, HasAttrs, edit::AstNodeEdit},
};

use crate::{AssistContext, AssistId, Assists, utils::test_related_attribute_syn};

// Assist: toggle_ignore
//
// Adds `#[ignore]` attribute to the test.
//
// ```
// $0#[test]
// fn arithmetics {
//     assert_eq!(2 + 2, 5);
// }
// ```
// ->
// ```
// #[test]
// #[ignore]
// fn arithmetics {
//     assert_eq!(2 + 2, 5);
// }
// ```
pub(crate) fn toggle_ignore(acc: &mut Assists, ctx: &AssistContext<'_, '_>) -> Option<()> {
    let attr: ast::Attr = ctx.find_node_at_offset()?;
    let func = attr.syntax().parent().and_then(ast::Fn::cast)?;
    let attr = test_related_attribute_syn(&func)?;
    let indent = attr.indent_level();

    match has_ignore_attribute(&func) {
        None => acc.add(
            AssistId::refactor("toggle_ignore"),
            "Ignore this test",
            attr.syntax().text_range_without_outer_trivia(),
            |builder| {
                let end = attr
                    .syntax()
                    .last_non_trivia_token()
                    .and_then(|it| {
                        it.trailing_trivia().take_while(|it| it.kind() != NEWLINE).last()
                    })
                    .map_or(attr.syntax().text_range_without_outer_trivia().end(), |it| {
                        it.text_range().end()
                    });
                builder.insert(end, format!("\n{indent}#[ignore]"))
            },
        ),
        Some(ignore_attr) => acc.add(
            AssistId::refactor("toggle_ignore"),
            "Re-enable this test",
            ignore_attr.syntax().text_range_without_outer_trivia(),
            |builder| {
                let range = ignore_attr.syntax().text_range_without_outer_trivia();
                let end = ignore_attr
                    .syntax()
                    .last_non_trivia_token()
                    .and_then(|last| {
                        let next = last.next_non_trivia_token()?;
                        let comment = last
                            .trailing_trivia()
                            .chain(next.leading_trivia())
                            .find(|it| it.kind() == COMMENT);
                        Some(
                            comment.map_or(next.text_range().start(), |it| it.text_range().start()),
                        )
                    })
                    .unwrap_or(range.end());
                builder.delete(TextRange::new(range.start(), end));
            },
        ),
    }
}

fn has_ignore_attribute(fn_def: &ast::Fn) -> Option<ast::Attr> {
    fn_def.attrs().find(|attr| {
        attr.path().is_some_and(|it| it.syntax().text_without_outer_trivia() == "ignore")
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn test_base_case() {
        check_assist(
            toggle_ignore,
            r#"
            mod indent {
                #[test$0]
                fn test() {}
            }
            "#,
            r#"
            mod indent {
                #[test]
                #[ignore]
                fn test() {}
            }
            "#,
        )
    }

    #[test]
    fn test_unignore() {
        check_assist(
            toggle_ignore,
            r#"
            mod indent {
                #[test$0]
                #[ignore]
                fn test() {}
            }
            "#,
            r#"
            mod indent {
                #[test]
                fn test() {}
            }
            "#,
        )
    }
}
