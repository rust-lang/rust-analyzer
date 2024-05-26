use ide_db::base_db::AnchoredPathBuf;
use stdx::to_lower_snake_case;
use syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode,
};

use crate::{utils::find_struct_impl, AssistContext, AssistId, AssistKind, Assists};

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

    let impl_def = find_struct_impl(ctx, &adt, &[]).flatten();
    let target = adt.syntax().text_range();

    acc.add(
        AssistId("move_definition_to_file", AssistKind::RefactorExtract),
        "Extract definition to file",
        target,
        |builder| {
            let module_name = to_lower_snake_case(&adt_name.text());
            let path = format!("./{}.rs", module_name);

            let mut content = adt_text.to_string();
            if let Some(impl_def) = impl_def {
                content.push_str("\n\n");
                content.push_str(&impl_def.syntax().text().to_string());
                builder.delete(impl_def.syntax().text_range());
            }

            let visibility = adt.visibility().map(|v| v.to_string() + " ");
            let visibility = visibility.as_deref().unwrap_or("");
            let mod_and_use_declaration = format!(
                "{}mod {};\n{}use {}::*;",
                visibility, module_name, visibility, module_name
            );
            builder.replace(target, mod_and_use_declaration);

            let dst = AnchoredPathBuf { anchor: ctx.file_id(), path };
            builder.create_file(dst, content);
        },
    )
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
