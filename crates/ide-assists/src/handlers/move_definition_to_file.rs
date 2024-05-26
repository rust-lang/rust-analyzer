use std::iter;

use hir::HasAttrs;
use ide_db::base_db::AnchoredPathBuf;
use itertools::Itertools;
use stdx::{format_to, to_lower_snake_case};
use syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode, SmolStr,
};

use crate::{AssistContext, AssistId, AssistKind, Assists};

fn find_all_impls(ctx: &AssistContext<'_>, adt: &ast::Adt) -> Vec<ast::Impl> {
    let db = ctx.db();
    let module = match adt.syntax().parent() {
        Some(module) => module,
        None => return Vec::new(),
    };

    let struct_def = match ctx.sema.to_def(adt) {
        Some(def) => def,
        None => return Vec::new(),
    };

    module
        .descendants()
        .filter_map(ast::Impl::cast)
        .filter(|impl_blk| {
            let blk = match ctx.sema.to_def(impl_blk) {
                Some(def) => def,
                None => return false,
            };

            match blk.self_ty(db).as_adt() {
                Some(def) => def == struct_def,
                None => false,
            }
        })
        .collect()
}

// Assist: move_definition_to_file
//
// Moves the selected ADT and its impl to a separate file as a new module.
//
// ```
// struct $0Foo {
//     x: i32,
// }
// impl Foo {
//     fn new(x: i32) -> Self {
//         Self { x }
//     }
// }
// ```
// ->
// ```
// mod foo;
// use foo::*;
// ```
pub(crate) fn move_definition_to_file(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let adt = ctx.find_node_at_offset::<ast::Adt>()?;
    let adt_name = adt.name()?;
    let adt_text = adt.syntax().text().to_string();

    let impls = find_all_impls(ctx, &adt);
    let target = adt.syntax().text_range();

    let parent_module = ctx.sema.file_to_module_def(ctx.file_id()).unwrap();
    let module_ast = adt.syntax().ancestors().find_map(ast::Module::cast);

    acc.add(
        AssistId("move_definition_to_file", AssistKind::RefactorExtract),
        "Extract definition to file",
        target,
        |builder| {
            let module_name = to_lower_snake_case(&adt_name.text());
            let path = construct_path(ctx, &parent_module, module_ast, &module_name);

            let mut buf = adt_text.clone();
            for impl_def in impls {
                buf.push_str("\n\n");
                buf.push_str(&impl_def.syntax().text().to_string());
                builder.delete(impl_def.syntax().text_range());
            }

            let visibility = adt.visibility().map_or_else(String::new, |v| format!("{} ", v));
            let mod_and_use_declaration = format!(
                "{vis}mod {name};\n{vis}use {name}::*;",
                vis = visibility,
                name = module_name
            );
            builder.replace(target, mod_and_use_declaration);

            let dst = AnchoredPathBuf { anchor: ctx.file_id(), path: path.display().to_string() };
            builder.create_file(dst, buf);
        },
    )
}

use std::path::PathBuf;

fn construct_path(
    ctx: &AssistContext<'_>,
    parent_module: &hir::Module,
    module_ast: Option<ast::Module>,
    module_name: &str,
) -> PathBuf {
    let mut path = PathBuf::from("./");
    let db = ctx.db();
    if let Some(name) = parent_module.name(db) {
        if !parent_module.is_mod_rs(db)
            && parent_module.attrs(db).by_key("path").string_value_unescape().is_none()
        {
            path.push(name.display(db).to_string());
        }
    }

    let segments = iter::successors(module_ast, |module| module.parent())
        .filter_map(|it| it.name())
        .map(|name| SmolStr::from(name.text().trim_start_matches("r#")))
        .collect::<Vec<_>>();

    for segment in segments.into_iter().rev() {
        path.push(segment.as_str());
    }
    path.push(format!("{}.rs", module_name));
    path
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn extract_struct_without_impls() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
struct $0Foo {
    x: i32,
}
"#,
            r#"
//- /main.rs
mod foo;
use foo::*;
//- /foo.rs
struct Foo {
    x: i32,
}"#,
        );
    }

    #[test]
    fn extract_struct_with_pub_mod() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
pub(crate) struct $0Foo {
    x: i32,
}
"#,
            r#"
//- /main.rs
pub(crate) mod foo;
pub(crate) use foo::*;
//- /foo.rs
pub(crate) struct Foo {
    x: i32,
}"#,
        );
    }

    #[test]
    fn extract_struct_with_impl() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
#[derive(Debug)]
struct $0FooBar {
    x: i32,
}

impl FooBar {
    fn new(x: i32) -> Self {
        Self { x }
    }
}
"#,
            r#"
//- /main.rs
mod foo_bar;
use foo_bar::*;


//- /foo_bar.rs
#[derive(Debug)]
struct FooBar {
    x: i32,
}

impl FooBar {
    fn new(x: i32) -> Self {
        Self { x }
    }
}"#,
        );
    }

    #[test]
    fn extract_struct_from_services_file() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
mod services;
//- /services/mod.rs
struct $0Foo {
    id: u32,
}

impl Foo {
    fn new(id: u32) -> Self {
        Self { id }
    }
}
"#,
            r#"
//- /services/mod.rs
mod foo;
use foo::*;


//- /services/foo.rs
struct Foo {
    id: u32,
}

impl Foo {
    fn new(id: u32) -> Self {
        Self { id }
    }
}"#,
        );
    }

    #[test]
    fn extract_nested_struct() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
mod foo {
    mod bar {
        struct $0Baz {
            y: i32,
        }

        impl Baz {
            fn new(y: i32) -> Self {
                Self { y }
            }
        }
    }
}
"#,
            r#"
//- /main.rs
mod foo {
    mod bar {
        mod baz;
use baz::*;

        
    }
}
//- /foo/bar/baz.rs
struct Baz {
            y: i32,
        }

impl Baz {
            fn new(y: i32) -> Self {
                Self { y }
            }
        }"#,
        );
    }

    #[test]
    fn extract_struct_with_trait_impl() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
struct $0Foo {
    x: i32,
}

impl Foo {
    fn new(x: i32) -> Self {
        Self { x }
    }
}

impl std::fmt::Display for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Foo: {}", self.x)
    }
}
"#,
            r#"
//- /main.rs
mod foo;
use foo::*;




//- /foo.rs
struct Foo {
    x: i32,
}

impl Foo {
    fn new(x: i32) -> Self {
        Self { x }
    }
}

impl std::fmt::Display for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Foo: {}", self.x)
    }
}"#,
        );
    }

    #[test]
    fn extract_enum() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
enum $0MyEnum {
    Variant1,
    Variant2(i32),
}
"#,
            r#"
//- /main.rs
mod my_enum;
use my_enum::*;
//- /my_enum.rs
enum MyEnum {
    Variant1,
    Variant2(i32),
}"#,
        );
    }

    #[test]
    fn extract_enum_with_impl() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
enum $0MyEnum {
    Variant1,
    Variant2(i32),
}

impl MyEnum {
    fn new_variant2(value: i32) -> Self {
        MyEnum::Variant2(value)
    }
}
"#,
            r#"
//- /main.rs
mod my_enum;
use my_enum::*;


//- /my_enum.rs
enum MyEnum {
    Variant1,
    Variant2(i32),
}

impl MyEnum {
    fn new_variant2(value: i32) -> Self {
        MyEnum::Variant2(value)
    }
}"#,
        );
    }

    #[test]
    fn extract_enum_without_variants() {
        check_assist(
            move_definition_to_file,
            r#"
//- /main.rs
enum $0EmptyEnum {}
"#,
            r#"
//- /main.rs
mod empty_enum;
use empty_enum::*;
//- /empty_enum.rs
enum EmptyEnum {}"#,
        );
    }

    #[test]
    fn no_struct_or_trait_selected() {
        check_assist_not_applicable(
            move_definition_to_file,
            r#"
//- /main.rs
fn $0foo() {}
"#,
        );
    }
}
