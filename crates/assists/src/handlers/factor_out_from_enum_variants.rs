use std::iter;

use syntax::{
    algo::SyntaxRewriter,
    ast::GenericParamsOwner,
    ast::{self, make, NameOwner, VisibilityOwner},
    AstNode,
};

use crate::{
    assist_context::{AssistContext, Assists},
    utils::existing_definition,
    AssistId, AssistKind,
};

// Assist: factor_out_from_enum_variants
//
// Factors out common fields from enum variants.
//
// ```
// enum A {<|>
//     One(u32, String),
//     Two(u32, bool),
// }
// ```
// ->
// ```
// struct A {
//     enum_field_0: u32,
//     enum_a: EnumA,
// }
//
// enum EnumA {
//     One(String),
//     Two(bool),
// }
// ```
pub(crate) fn factor_out_from_enum_variants(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let enum_ = ctx.find_node_at_offset::<ast::Enum>()?;
    let enum_vis = enum_.visibility();
    let enum_name = enum_.name()?;

    // FIXME: Support generic enums
    if enum_.generic_param_list().is_some() {
        return None;
    }

    let new_ty = make::name(&format!("Enum{}", enum_name.text()));
    let db = ctx.db();
    let module = ctx.sema.to_def(&enum_)?.module(db);
    if existing_definition(db, &new_ty, module) {
        return None;
    }

    let mut variants = enum_.variant_list()?.variants();
    let first_variant = &variants.next()?;
    let common_fields = match first_variant.field_list()? {
        // FIXME: Support record fields.
        ast::FieldList::RecordFieldList(_) => return None,
        ast::FieldList::TupleFieldList(f) => common_tuple_fields(f, variants)?,
    };

    let target = enum_.syntax().text_range();
    acc.add(
        AssistId("factor_out_from_enum_variants", AssistKind::RefactorRewrite),
        "Factor out from enum variants",
        target,
        // FIXME: Find and update creation of the enum.
        // FIXME: Find and update usage of the enum.
        |builder| {
            let fields = make::record_field_list(
                common_fields
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| {
                        let name = make::name(&format!("enum_field_{}", i));
                        make::record_field(enum_vis.clone(), name, make::ty(ty))
                    })
                    .chain(iter::once({
                        let ty = make::ty(new_ty.text());
                        let name = make::name(&stdx::to_lower_snake_case(new_ty.text()));
                        make::record_field(enum_vis.clone(), name, ty)
                    })),
            );

            builder.edit_file(ctx.frange.file_id);
            let mut rewriter = SyntaxRewriter::default();

            // FIXME: Support generic enums
            let strukt = make::struct_(enum_vis, enum_name, None, fields.into());
            rewriter.insert_before(enum_.syntax(), strukt.syntax());
            rewriter.insert_before(enum_.syntax(), &make::tokens::blank_line());
            update_tuple_enum(&mut rewriter, &enum_, common_fields.len());

            builder.rewrite(rewriter);
        },
    );

    None
}

fn common_tuple_fields(
    first_variant: ast::TupleFieldList,
    variants: ast::AstChildren<ast::Variant>,
) -> Option<Vec<String>> {
    let mut common: Vec<_> = first_variant
        .fields()
        // FIXME(rust/#68537): Replace the `.scan()`s with .map_while after it's stabilized.
        // This .scan(..) is equivalent to .map_while(|field| field.ty().map..)
        .scan((), |(), field| field.ty().map(|ty| ty.syntax().text().to_string()))
        .collect();

    for variant in variants {
        let types = match variant.field_list()? {
            ast::FieldList::TupleFieldList(f) => {
                f.fields().scan((), |(), field| field.ty().map(|ty| ty.syntax().text().to_string()))
            }
            _ => return None,
        };

        let in_common = common.iter().zip(types).take_while(|(a, b)| *a == b).count();
        common.drain(in_common..);
    }

    if common.is_empty() {
        None
    } else {
        Some(common)
    }
}

fn update_tuple_enum(
    rewriter: &mut SyntaxRewriter,
    enum_: &ast::Enum,
    n_common_fields: usize,
) -> Option<()> {
    let name = enum_.name()?;
    let new_name = make::name(&format!("Enum{}", name.text()));
    rewriter.replace(name.syntax(), new_name.syntax());

    for variant in enum_.variant_list()?.variants() {
        let mut fields = match variant.field_list()? {
            ast::FieldList::TupleFieldList(f) => f.fields().skip(n_common_fields).peekable(),
            _ => return None,
        };

        let unique_variant = make::variant(
            variant.name()?,
            match fields.peek() {
                Some(_) => Some(make::tuple_field_list(fields).into()),
                _ => None,
            },
        );
        rewriter.replace(variant.syntax(), unique_variant.syntax());
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn factor_out_from_enum_variant_works() {
        check_assist(
            factor_out_from_enum_variants,
            r#"
pub enum Data {<|>
    A(usize, [String; 2], bool),
    B(usize, [String; 2]),
    C(usize, [String; 2], u32, i16),
}"#,
            r#"
pub struct Data {
    pub enum_field_0: usize,
    pub enum_field_1: [String; 2],
    pub enum_data: EnumData,
}

pub enum EnumData {
    A(bool),
    B,
    C(u32, i16),
}"#,
        );
    }
}
