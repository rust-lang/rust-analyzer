use syntax::{
    ast::{self, VisibilityOwner},
    AstNode, TextSize,
};
use test_utils::mark;

use crate::{utils::vis_offset, AssistContext, AssistId, AssistKind, Assists};

// Assist: change_fields_visibility
//
// Adds visibility for each field inside public struct.
//
// ```
// pub struct <|>Foo {
//    bar: usize,
//    test: String
// }
// ```
// ->
// ```
// pub struct Foo {
//    pub bar: usize,
//    pub test: String
// }
// ```
pub(crate) fn change_fields_visibility(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let strukt = ctx.find_node_at_offset::<ast::Struct>()?;
    let vis = strukt.visibility()?;
    if vis.pub_token().is_none() {
        return None;
    }

    return add_vis_fields(acc, strukt, vis);
}

fn add_vis_fields(
    acc: &mut Assists,
    strukt: ast::Struct,
    strukt_vis: ast::Visibility,
) -> Option<()> {
    let fields = strukt.field_list()?;

    let offsets: Vec<TextSize> = match fields {
        ast::FieldList::RecordFieldList(record_field_list) => record_field_list
            .fields()
            .into_iter()
            .filter_map(|field| match field.visibility() {
                Some(_) => None,
                None => Some(vis_offset(field.syntax())),
            })
            .collect(),
        ast::FieldList::TupleFieldList(tuple_field_list) => tuple_field_list
            .fields()
            .into_iter()
            .filter_map(|field| match field.visibility() {
                Some(_) => None,
                None => Some(vis_offset(field.syntax())),
            })
            .collect(),
    };

    if offsets.is_empty() {
        mark::hit!(change_fields_visibility_all_fields_already_public);
        return None;
    }

    let new_vis = match strukt_vis.crate_token() {
        Some(_) => "pub(crate) ",
        None => "pub ",
    };

    acc.add(
        AssistId("change_fields_visibility", AssistKind::RefactorRewrite),
        "Make all fields public",
        strukt.syntax().text_range(),
        |edit| {
            for offset in offsets {
                edit.insert(offset, new_vis);
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn change_fields_visibility_for_pub() {
        check_assist(
            change_fields_visibility,
            r"pub struct S<|> { field: u32 }",
            r"pub struct S { pub field: u32 }",
        );
    }

    #[test]
    fn change_fields_visibility_for_pub_several_fields() {
        check_assist(
            change_fields_visibility,
            r"pub struct <|>Foo {
                field: u32,
                bar: usize
            }",
            r"pub struct Foo {
                pub field: u32,
                pub bar: usize
            }",
        );
    }

    #[test]
    fn change_fields_visibility_for_pub_crate() {
        check_assist(
            change_fields_visibility,
            r"pub(crate) struct <|>Foo {
                field: u32,
                bar: usize
            }",
            r"pub(crate) struct Foo {
                pub(crate) field: u32,
                pub(crate) bar: usize
            }",
        );
    }

    #[test]
    fn change_fields_visibility_for_pub_already_pub_fields() {
        check_assist(
            change_fields_visibility,
            r"pub struct <|>Foo {
                pub(crate) field: u32,
                bar: usize
            }",
            r"pub struct Foo {
                pub(crate) field: u32,
                pub bar: usize
            }",
        );
    }

    #[test]
    fn change_fields_visibility_for_pub_already_all_pub_fields() {
        mark::check!(change_fields_visibility_all_fields_already_public);
        check_assist_not_applicable(
            change_fields_visibility,
            r"pub struct <|>Foo {
                pub(crate) field: u32,
                pub bar: usize
            }",
        );
    }

    #[test]
    fn change_fields_visibility_with_private_struct() {
        check_assist_not_applicable(
            change_fields_visibility,
            r"struct <|>Foo {
                field: u32,
                bar: usize
            }",
        )
    }
}
