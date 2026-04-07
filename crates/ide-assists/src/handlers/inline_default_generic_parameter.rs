use crate::{AssistContext, AssistId, Assists};
use syntax::{
    AstNode, TextRange,
    ast::{self, HasGenericArgs, HasName},
};

// Assist: inline_default_generic_parameter
//
// Inlines a default generic parameter to all of its usages.
//
// ```
// struct Foo<T = i32$0>(T);
// impl Foo { }
// ```
// ->
// ```
// struct Foo<T>(T);
// impl Foo<i32> { }
// ```
pub(crate) fn inline_default_generic_parameter(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let type_param = ctx.find_node_at_offset::<ast::TypeParam>()?;
    let default_type = type_param.default_type()?;

    let generic_param_list = type_param.syntax().parent().and_then(ast::GenericParamList::cast)?;
    let item = generic_param_list.syntax().parent().and_then(ast::Item::cast)?;

    let adt = ast::Adt::cast(item.syntax().clone())?;
    let adt_name = adt.name()?;

    acc.add(
        AssistId::quick_fix("inline_default_generic_parameter"),
        "Inline default generic parameter",
        type_param.syntax().text_range(),
        |builder| {
            if let Some(name) = type_param.name() {
                let start = name.syntax().text_range().end();
                let end = default_type.syntax().text_range().end();
                builder.delete(TextRange::new(start, end));
            }

            let default_text = default_type.syntax().text().to_string();
            let insert_text = format!("<{}>", default_text);

            let source_file = ctx.sema.parse(ctx.file_id());

            for path_segment in
                source_file.syntax().descendants().filter_map(ast::PathSegment::cast)
            {
                if let Some(name_ref) = path_segment.name_ref()
                    && name_ref.text() == adt_name.text()
                {
                    let is_type =
                        path_segment.syntax().ancestors().find_map(ast::PathType::cast).is_some();
                    let has_no_generics = path_segment.generic_arg_list().is_none();

                    if is_type && has_no_generics {
                        builder
                            .insert(path_segment.syntax().text_range().end(), insert_text.clone());
                    }
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn test_inline_default_generic_parameter() {
        check_assist(
            inline_default_generic_parameter,
            r#"
struct Foo<T = i32$0>(T);
impl Foo {
    fn foo(&self) {}
}
fn main() {
    let _ = Foo::foo;
    let _ = <Foo>::foo;
}
"#,
            r#"
struct Foo<T>(T);
impl Foo<i32> {
    fn foo(&self) {}
}
fn main() {
    let _ = Foo::foo;
    let _ = <Foo<i32>>::foo;
}
"#,
        );
    }

    #[test]
    fn test_not_applicable_without_default() {
        check_assist_not_applicable(
            inline_default_generic_parameter,
            r#"
struct Foo<T$0>(T);
"#,
        );
    }

    #[test]
    fn test_inline_in_function_signatures() {
        check_assist(
            inline_default_generic_parameter,
            r#"
struct CustomBox<T = i32$0>(T);

fn process(b: CustomBox) -> CustomBox {
    b
}
"#,
            r#"
struct CustomBox<T>(T);

fn process(b: CustomBox<i32>) -> CustomBox<i32> {
    b
}
"#,
        );
    }

    #[test]
    fn test_works_on_enums() {
        check_assist(
            inline_default_generic_parameter,
            r#"
enum Result<T = i32$0> {
    Ok(T),
    Err(()),
}

impl Result {
    fn is_ok(&self) -> bool { true }
}
"#,
            r#"
enum Result<T> {
    Ok(T),
    Err(()),
}

impl Result<i32> {
    fn is_ok(&self) -> bool { true }
}
"#,
        );
    }
}
