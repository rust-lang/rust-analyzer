use std::iter;

use ast::edit::IndentLevel;
use ide_db::{defs::Definition, search::Reference};
use rustc_hash::{FxHashMap, FxHashSet};
use syntax::{
    algo::{self, SyntaxRewriter},
    ast::edit::AstNodeEdit,
    ast::ArgListOwner,
    ast::GenericParamsOwner,
    ast::{self, make, NameOwner, VisibilityOwner},
    AstNode, SourceFile,
};

use crate::{
    assist_context::{AssistContext, Assists},
    utils, AssistId, AssistKind,
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
    let enum_hir = ctx.sema.to_def(&enum_)?;
    let module = enum_hir.module(db);
    if utils::existing_definition(db, &new_ty, module) {
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
        // FIXME: Find and update usage of the enum.
        |builder| {
            let new_fields = make::record_field_list(
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

            let usages = Definition::ModuleDef(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)))
                .usages(&ctx.sema)
                .all();

            let mut visited_modules_set = FxHashSet::default();
            visited_modules_set.insert(module);
            let mut rewriters = FxHashMap::default();
            for reference in usages {
                let rewriter = rewriters
                    .entry(reference.file_range.file_id)
                    .or_insert_with(SyntaxRewriter::default);
                let source_file = ctx.sema.parse(reference.file_range.file_id);
                update_constructor(
                    rewriter,
                    reference,
                    &source_file,
                    new_ty.text(),
                    &new_fields,
                    common_fields.len(),
                );
            }

            let mut rewriter = rewriters.remove(&ctx.frange.file_id).unwrap_or_default();
            for (file_id, rewriter) in rewriters {
                // FIXME(#6465): Currently broken for multiple files.
                builder.edit_file(file_id);
                builder.rewrite(rewriter);
            }
            builder.edit_file(ctx.frange.file_id);

            // FIXME: Support generic enums
            let strukt = make::struct_(enum_vis, enum_name.clone(), None, new_fields.into());

            let indent_level = IndentLevel::from_node(enum_.syntax());
            rewriter.insert_before(enum_.syntax(), strukt.indent(indent_level).syntax());
            // Just using make::tokens::blank_line() won't work because we're inserting between the
            // enum_ and its indentation.
            let ws = make::tokens::whitespace(&format!("\n\n{}", indent_level));
            rewriter.insert_before(enum_.syntax(), &ws);

            update_tuple_enum(&mut rewriter, &enum_, &enum_name, &new_ty, common_fields.len());
            builder.rewrite(rewriter);
        },
    );

    Some(())
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

fn update_constructor(
    rewriter: &mut SyntaxRewriter,
    reference: Reference,
    source_file: &SourceFile,
    new_enum_name: &str,
    struct_fields: &ast::RecordFieldList,
    n_common_fields: usize,
) -> Option<()> {
    // FIXME: check for ast::RecordExpr
    let path_expr = algo::find_node_at_offset::<ast::PathExpr>(
        source_file.syntax(),
        reference.file_range.range.start(),
    )?;
    let path = path_expr.path()?;
    let (old_enum_path, variant) = (path.qualifier()?, path.segment()?);

    let call = path_expr.syntax().parent().and_then(ast::CallExpr::cast)?;
    let args = call.arg_list()?.args().collect::<Vec<_>>();
    let (common, unique) = args.split_at(n_common_fields);

    let new_enum_path = make::expr_path(make::path_qualified(
        make::path_unqualified(make::path_segment(make::name_ref(new_enum_name))),
        variant,
    ));
    let enum_constructor = if unique.is_empty() {
        new_enum_path
    } else {
        make::expr_call(new_enum_path, make::arg_list(unique.iter().cloned()))
    };

    let enum_record_field = make::record_expr_field(
        make::name_ref(&stdx::to_lower_snake_case(new_enum_name)),
        Some(enum_constructor),
    );
    let name_refs =
        struct_fields.fields().filter_map(|it| it.name().map(|it| make::name_ref(it.text())));
    // Because there's an extra field containing an enum, name_refs.count() == n_common_fields + 1
    debug_assert_eq!(name_refs.clone().count(), n_common_fields + 1);

    let record_expr_field_list = make::record_expr_field_list(
        name_refs
            .zip(common.iter().cloned())
            .map(|(name_ref, expr)| make::record_expr_field(name_ref, Some(expr)))
            .chain(iter::once(enum_record_field)),
    );
    let constructor = make::record_expr(old_enum_path, record_expr_field_list);

    let level = IndentLevel::from_node(call.syntax());
    rewriter.replace(call.syntax(), constructor.indent(level).syntax());

    Some(())
}

fn update_tuple_enum(
    rewriter: &mut SyntaxRewriter,
    enum_: &ast::Enum,
    enum_name: &ast::Name,
    new_enum_name: &ast::Name,
    n_common_fields: usize,
) -> Option<()> {
    rewriter.replace(enum_name.syntax(), new_enum_name.syntax());

    for variant in enum_.variant_list()?.variants() {
        let mut fields = match variant.field_list()? {
            ast::FieldList::TupleFieldList(f) => f.fields().skip(n_common_fields).peekable(),
            _ => return None,
        };

        let unique_variant = make::variant(
            variant.name()?,
            // Trying to instead use .map() here makes borrowck very angry :(
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
    fn factor_out_works() {
        check_assist(
            factor_out_from_enum_variants,
            r#"
pub enum Data {<|>
    A(usize, [String; 2], bool),
    B(usize, [String; 2]),
    C(usize, [String; 2], u32, i16),
}
"#,
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
}
"#,
        );
    }

    #[test]
    fn factor_out_update_construction_works() {
        check_assist(
            factor_out_from_enum_variants,
            r#"
mod also_test_indentation {
    enum A {<|>
        One(String, bool, usize, f64),
        Two(String, bool,),
    }

    fn func() {
        let _one = A::One("hi".into(), true, 42, 12.);
        let _two = A::Two("hi".into(), true);
    }
}
"#,
            r#"
mod also_test_indentation {
    struct A {
        enum_field_0: String,
        enum_field_1: bool,
        enum_a: EnumA,
    }

    enum EnumA {
        One(usize, f64),
        Two,
    }

    fn func() {
        let _one = A {
            enum_field_0: "hi".into(),
            enum_field_1: true,
            enum_a: EnumA::One(42, 12.),
        };
        let _two = A {
            enum_field_0: "hi".into(),
            enum_field_1: true,
            enum_a: EnumA::Two,
        };
    }
}
"#,
        );
    }

    #[test]
    fn factor_out_update_match_usage_works() {
        check_assist(
            factor_out_from_enum_variants,
            r#"
enum A {
    One(String, usize),<|>
    Two(String),
}

fn func(it: A) {
    match it {
        A::One(text, n) => {
            println!("{}", text.repeat(n));
        }
        A::Two(text) => println!("{}", text),
    }
}
"#,
            r#"
struct A {
    enum_field_0: String,
    enum_a: EnumA,
}

enum EnumA {
    One(usize),
    Two,
}

fn func(it: A) {
    match it.enum_a {
        EnumA::One(n) => {
            println!("{}", it.enum_field_0.repeat(n));
        }
        EnumA::Two => println!("{}", it.enum_field_0),
    }
}
"#,
        );
    }
}
