use crate::{Assist, AssistCtx, AssistId, TextUnit};
use hir::{db::HirDatabase, HasSource};
use ra_db::FilePosition;
use ra_syntax::ast::{self, AstNode, NameOwner, TypeAscriptionOwner};

pub(crate) fn add_new(mut ctx: AssistCtx<impl HirDatabase>) -> Option<Assist> {
    let impl_node = ctx.node_at_offset::<ast::ImplBlock>()?;

    let struct_def = {
        let file_id = ctx.frange.file_id;
        let position = FilePosition { file_id, offset: impl_node.syntax().text_range().start() };
        let analyzer = hir::SourceAnalyzer::new(ctx.db, position.file_id, impl_node.syntax(), None);
        resolve_target_struct_def(ctx.db, &analyzer, &impl_node)?
    };

    let struct_name = struct_def.name()?;
    let mut children = struct_def.syntax().children();
    let field_def_list = children.find_map(ast::NamedFieldDefList::cast)?;

    ctx.add_action(AssistId("add_new"), "add new", |edit| {
        let mut buf = String::new();
        buf.push_str("fn new(");
        let fields: Vec<_> = field_def_list.fields().filter_map(|f| {
            Some((f.name()?.syntax().text(), f.ascribed_type()?.syntax().text()))
        }).collect();

        let mut first = true;
        for field in &fields {
            if !first {
                buf.push_str(", ");
            }
            buf.push_str(&format!("{}: {}", field.0, field.1));
            first = false;
        }
        buf.push_str(") -> Self {\n");
        buf.push_str(&format!("        {} {{ ", struct_name.syntax().text()));
        let mut first = true;
        for field in &fields {
            if !first {
                buf.push_str(", ");
            }
            buf.push_str(&format!("{}", field.0));
            first = false;
        }
        buf.push_str(" }\n");
        buf.push_str("    }");

        // TODO: not this!
        let insert_position = impl_node.syntax().text_range().end() - TextUnit::from_usize(2);
        let insert_len = buf.len();
        edit.target(impl_node.syntax().text_range());
        edit.insert(insert_position, buf);
        edit.set_cursor(insert_position + TextUnit::from_usize(insert_len));
    });

    ctx.build()
}

fn resolve_target_struct_def(
    db: &impl HirDatabase,
    analyzer: &hir::SourceAnalyzer,
    impl_block: &ast::ImplBlock,
) -> Option<ast::StructDef> {
    let ast_path = impl_block
        .target_type()
        .map(|it| it.syntax().clone())
        .and_then(ast::PathType::cast)?
        .path()?;

    match analyzer.resolve_path(db, &ast_path) {
        Some(hir::PathResolution::Def(hir::ModuleDef::Struct(def))) => Some(def.source(db).ast),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::{check_assist, check_assist_not_applicable};

    // TODO: handle, and add tests for:
    //  type parameters
    //  lifetime parameters
    //  non-empty impl blocks

    #[test]
    fn test_add_new() {
        check_assist(
            add_new,
            r#"
struct Foo {
    x: i32,
    y: String
}

impl Foo {
    <|>
}"#,
            r#"
struct Foo {
    x: i32,
    y: String
}

impl Foo {
    fn new(x: i32, y: String) -> Self {
        Foo { x, y }
    }<|>
}"#);
    }

     #[test]
    fn test_add_new_ignores_enums() {
        check_assist_not_applicable(
            add_new,
            r#"
enum Foo {
    X(i32),
    Y(String)
}

impl Foo {
    <|>
}"#
        );
    }
}
