use hir::{AsAssocItem, HasSource};
use ide_db::assists::AssistId;
use syntax::{
    AstNode,
    NodeOrToken::Token,
    SyntaxElement,
    ast::{
        self, AnyHasName, HasDocComments, HasName,
        edit::{AstNodeEdit, IndentLevel},
        make,
    },
    syntax_editor::Position,
};

use crate::assist_context::{AssistContext, Assists};

// Assist: generate_documentation_from_trait
//
// Generate documents from items defined in the trait.
//
// ```
// trait Foo {
//     /// some docs
//     fn foo(&self);
// }
// impl Foo for () {
//     fn $0foo(&self) {}
// }
// ```
// ->
// ```
// trait Foo {
//     /// some docs
//     fn foo(&self);
// }
// impl Foo for () {
//     /// some docs
//     fn foo(&self) {}
// }
// ```
pub(crate) fn generate_documentation_from_trait(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let name = ctx.find_node_at_offset::<ast::Name>()?;
    let ast_func = name.syntax().parent().and_then(ast::Fn::cast)?;
    if ast_func.doc_comments().next().is_some() {
        return None;
    }
    let assoc_item = ctx.sema.to_def(&ast_func)?.as_assoc_item(ctx.db())?;
    let trait_ = assoc_item.implemented_trait(ctx.db())?.source(ctx.db())?.value;
    let origin_item = trait_.assoc_item_list()?.assoc_items().find(|it| {
        AnyHasName::cast(it.syntax().clone())
            .and_then(|it| it.name())
            .is_some_and(|it| it.text() == name.text())
    })?;
    let _first = origin_item.doc_comments().next()?;

    let comments = origin_item.doc_comments();
    let indent = ast_func.indent_level();
    let origin_indent = origin_item.indent_level();

    acc.add(
        AssistId::generate("generate_documentation_from_trait"),
        "Generate a documentation from trait",
        ast_func.syntax().text_range(),
        |builder| {
            let mut edit = builder.make_editor(ast_func.syntax());

            let comments = comments
                .flat_map(|doc| {
                    generate_docs(doc.doc_comment().unwrap_or(""), indent, origin_indent)
                })
                .collect();
            edit.insert_all(Position::before(ast_func.syntax()), comments);

            builder.add_file_edits(ctx.vfs_file_id(), edit);
        },
    )
}

fn generate_docs(doc: &str, indent: IndentLevel, origin_indent: IndentLevel) -> Vec<SyntaxElement> {
    let ws = format!("\n{indent}");
    let trim_indent = origin_indent.to_string();

    doc.trim_end()
        .split('\n')
        .flat_map(|doc| {
            let trimmed_doc = line_doc(&trim_indent, doc);
            [make::tokens::doc_comment(&trimmed_doc), make::tokens::whitespace(&ws)]
        })
        .map(|it| Token(it).into())
        .collect()
}

fn line_doc<'a>(trim_indent: &str, line: &'a str) -> String {
    let text = line.strip_prefix(trim_indent).unwrap_or(line.trim()).trim_end();
    if text.is_empty() { "///".to_owned() } else { format!("/// {text}") }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_generate_documentation_from_trait() {
        check_assist(
            generate_documentation_from_trait,
            r#"
trait Foo {
    /// some docs
    ///
    /// # Examples
    /// ...
    fn foo(&self);
}
impl Foo for () {
    fn $0foo(&self) {}
}
            "#,
            r#"
trait Foo {
    /// some docs
    ///
    /// # Examples
    /// ...
    fn foo(&self);
}
impl Foo for () {
    /// some docs
    ///
    /// # Examples
    /// ...
    fn foo(&self) {}
}
            "#,
        );
    }

    #[test]
    fn test_generate_documentation_from_trait_with_multi_line() {
        check_assist(
            generate_documentation_from_trait,
            r#"
trait Foo {
    /** some docs
    ...
    */
    fn foo(&self);
}
impl Foo for () {
    fn $0foo(&self) {}
}
            "#,
            r#"
trait Foo {
    /** some docs
    ...
    */
    fn foo(&self);
}
impl Foo for () {
    /// some docs
    /// ...
    fn foo(&self) {}
}
            "#,
        );
    }

    #[test]
    fn test_generate_documentation_from_trait_not_applicable_existing_doc() {
        check_assist_not_applicable(
            generate_documentation_from_trait,
            r#"
trait Foo {
    /// some docs
    ///
    /// # Examples
    /// ...
    fn foo(&self);
}
impl Foo for () {
    /// existing docs
    fn $0foo(&self) {}
}
            "#,
        );
    }
}
