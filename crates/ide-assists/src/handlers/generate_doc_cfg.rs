use ide_db::assists::GroupLabel;
use syntax::{
    NodeOrToken::{Node, Token},
    T,
    ast::{self, AstNode, edit_in_place::Indent, make},
    syntax_editor::Position,
};

use crate::{AssistContext, AssistId, Assists};

// Assist: generate_doc_cfg
//
// Generate a doc-cfg attribute from cfg attribute.
//
// ```
// #[$0cfg(unix)]
// pub struct Foo;
// ```
// ->
// ```
// #[cfg(unix)]
// #[cfg_attr(docsrs, doc(cfg(unix)))]
// pub struct Foo;
// ```
pub(crate) fn generate_doc_cfg(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let name = ctx.find_node_at_offset::<ast::Path>()?;
    let attr = ast::Attr::cast(name.syntax().parent()?.parent()?)?;
    let indent = attr.indent_level();

    if attr.simple_name()? != "cfg" {
        return None;
    }

    let target = attr.syntax().text_range();
    let group = GroupLabel("Generate doc-cfg".to_owned());

    for on in ["docsrs", "doc"] {
        let doc_cfg = generate_doc_cfg_attribute(&attr, on)?;
        acc.add_group(
            &group,
            AssistId::generate("generate_doc_cfg"),
            format!("Generate `{on}(docsrs, doc(cfg(...)))` from cfg"),
            target,
            |builder| {
                let mut edit = builder.make_editor(attr.syntax());

                edit.insert_all(
                    Position::after(attr.syntax()),
                    vec![
                        make::tokens::whitespace(&format!("\n{indent}")).into(),
                        doc_cfg.syntax().clone().into(),
                    ],
                );

                builder.add_file_edits(ctx.vfs_file_id(), edit);
            },
        );
    }
    Some(())
}

fn generate_doc_cfg_attribute(attr: &ast::Attr, on: &str) -> Option<ast::Attr> {
    let cfg = attr.token_tree()?;
    let is_inner_attr = attr.excl_token().is_some();

    let cfg_attr = make::meta_token_tree(
        ident_path("cfg_attr"),
        make::token_tree(
            T!['('],
            [
                Token(make::tokens::ident(on)),
                Token(make::token(T![,])),
                Token(make::tokens::single_space()),
                Token(make::tokens::ident("doc")),
                Node(make::token_tree(T!['('], [Token(make::tokens::ident("cfg")), Node(cfg)])),
            ],
        ),
    );

    if is_inner_attr {
        Some(make::attr_inner(cfg_attr).clone_for_update())
    } else {
        Some(make::attr_outer(cfg_attr).clone_for_update())
    }
}

fn ident_path(s: &str) -> ast::Path {
    make::path_from_segments([make::path_segment(make::name_ref(s))], false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::check_assist;

    #[test]
    fn test_generate_doc_cfg() {
        check_assist(
            generate_doc_cfg,
            r#"
                #[$0cfg(unix)]
                pub struct Foo;
            "#,
            r#"
                #[cfg(unix)]
                #[cfg_attr(docsrs, doc(cfg(unix)))]
                pub struct Foo;
            "#,
        );
    }

    #[test]
    fn test_generate_doc_cfg_in_mod() {
        check_assist(
            generate_doc_cfg,
            r#"
                mod foo {
                    #[$0cfg(unix)]
                    pub struct Foo;
                }
            "#,
            r#"
                mod foo {
                    #[cfg(unix)]
                    #[cfg_attr(docsrs, doc(cfg(unix)))]
                    pub struct Foo;
                }
            "#,
        );
    }
}
