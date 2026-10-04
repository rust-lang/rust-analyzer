use syntax::{
    AstNode,
    ast::{self, HasAttrs, make},
    syntax_editor::Position,
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

    match has_ignore_attribute(&func) {
        None => acc.add(
            AssistId::refactor("toggle_ignore"),
            "Ignore this test",
            attr.syntax().text_range_without_outer_trivia(),
            |builder| {
                let editor = builder.make_editor(attr.syntax());
                let make = editor.make();
                let ignore = make.attr_outer(make::meta_path(make.ident_path("ignore")));
                let ignore = make.with_elastic_line_break(ignore.syntax());
                editor.insert(Position::after(attr.syntax()), ignore);
                builder.add_file_edits(ctx.vfs_file_id(), editor);
            },
        ),
        Some(ignore_attr) => acc.add(
            AssistId::refactor("toggle_ignore"),
            "Re-enable this test",
            ignore_attr.syntax().text_range_without_outer_trivia(),
            |builder| {
                let editor = builder.make_editor(ignore_attr.syntax());
                editor.delete(ignore_attr.syntax());
                builder.add_file_edits(ctx.vfs_file_id(), editor);
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
