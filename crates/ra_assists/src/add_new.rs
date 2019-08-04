use crate::{Assist, AssistCtx, AssistId, TextRange, TextUnit};
use hir::{db::HirDatabase, HasSource};
use ra_db::FilePosition;
use ra_syntax::{
    ast::{self, AstNode, NameOwner, TypeAscriptionOwner},
    T
};

pub(crate) fn add_new(mut ctx: AssistCtx<impl HirDatabase>) -> Option<Assist> {
    let impl_node = ctx.node_at_offset::<ast::ImplBlock>()?;
    let item_list = impl_node.item_list()?;
    let items: Vec<_> = item_list.impl_items().collect();

    // Don't attempt to create a `new` function if one already exists.
    let fn_name = |item: &ast::ImplItem| {
        match item.kind() {
            ast::ImplItemKind::FnDef(def) => def.name(),
            _ => None
        }
    };
    if items.iter().filter_map(fn_name).any(|n| n.text() == "new") {
        return None;
    }

    // Find the type that this impl block is for, returning None if that type is not a struct.
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
        buf.push_str("\n    fn new(");
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
        buf.push_str(&format!("        {} {{", struct_name.syntax().text()));
        let mut first = true;
        for field in &fields {
            buf.push_str(if first { " " } else { ", " });
            buf.push_str(&format!("{}", field.0));
            first = false;
        }
        buf.push_str(" }\n");
        buf.push_str("    }\n");
        let mut roffset = 1;
        if items.len() > 0 {
            buf.push_str("\n    ");
            roffset += 5;
        }

        let l_paren = item_list.syntax().children_with_tokens().find(|it| it.kind() == T!['{']);
        if l_paren.is_none() {
            return;
        }
        let replace_start = l_paren.unwrap().text_range().start() + TextUnit::from_usize(1);
        let replace_end = if items.len() > 0 {
            items[0].syntax().text_range().start()
        } else {
            let r_paren = item_list.syntax().children_with_tokens().find(|it| it.kind() == T!['}']);
            if r_paren.is_none() {
                return;
            }
            r_paren.unwrap().text_range().start()
        };
        let replace_range = TextRange::from_to(replace_start, replace_end);

        let insert_len = buf.len();
        edit.target(impl_node.syntax().text_range());
        edit.replace(replace_range, buf);
        edit.set_cursor(replace_start + TextUnit::from_usize(insert_len - roffset));
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

    #[test]
    fn test_add_new_no_fields() {
        check_assist(
            add_new,
            r#"
struct Foo { }

impl Foo {
    <|>
}"#,
            r#"
struct Foo { }

impl Foo {
    fn new() -> Self {
        Foo { }
    }<|>
}"#
        );
    }

 #[test]
    fn test_add_new_with_no_whitespace_in_impl() {
        check_assist(
            add_new,
            r#"
struct Foo { }

impl Foo <|>{}"#,
            r#"
struct Foo { }

impl Foo {
    fn new() -> Self {
        Foo { }
    }<|>
}"#
        );
    }

    #[test]
    fn test_add_new_one_field() {
        check_assist(
            add_new,
            r#"
struct Foo {
    x: i32
}

impl Foo {
    <|>
}"#,
            r#"
struct Foo {
    x: i32
}

impl Foo {
    fn new(x: i32) -> Self {
        Foo { x }
    }<|>
}"#
        );
    }

    #[test]
    fn test_add_new_two_fields() {
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
}"#
        );
    }

    #[test]
    fn test_add_new_with_type_parameter() {
        check_assist(
            add_new,
            r#"
struct Foo<T> {
    x: T
}

impl<T> Foo<T> {
    <|>
}"#,
            r#"
struct Foo<T> {
    x: T
}

impl<T> Foo<T> {
    fn new(x: T) -> Self {
        Foo { x }
    }<|>
}"#
        );
    }

    #[test]
    fn test_add_new_with_lifetime_parameters() {
        check_assist(
            add_new,
            r#"
struct Foo<'a, 'b, T, U> {
    x: &'a T,
    y: &'b U
}

impl<'a, 'b, T, U> Foo<'a, 'b, T, U> {
    <|>
}"#,
            r#"
struct Foo<'a, 'b, T, U> {
    x: &'a T,
    y: &'b U
}

impl<'a, 'b, T, U> Foo<'a, 'b, T, U> {
    fn new(x: &'a T, y: &'b U) -> Self {
        Foo { x, y }
    }<|>
}"#
        );
    }

    #[test]
    fn test_add_new_into_non_empty_impl_block() {
        check_assist(
            add_new,
            r#"
struct Foo {
    x: i32
}

impl Foo {
    fn a() { }
<|>
    fn b() -> i32 {
        7
    }
}"#,
            r#"
struct Foo {
    x: i32
}

impl Foo {
    fn new(x: i32) -> Self {
        Foo { x }
    }<|>

    fn a() { }

    fn b() -> i32 {
        7
    }
}"#
        );
    }

    #[test]
    fn test_add_new_not_applicable_if_new_already_exists() {
        check_assist_not_applicable(
            add_new,
            r#"
struct Foo {
    x: i32,
    y: String
}

impl Foo {
    <|>
    fn new() { }
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
