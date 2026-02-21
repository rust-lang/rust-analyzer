use either::Either;
use ide_db::{defs::Definition, search::FileReference};
use syntax::{
    SyntaxKind,
    ast::{self, AstNode, HasGenericParams},
    match_ast,
};

use crate::{AssistContext, AssistId, Assists, assist_context::SourceChangeBuilder};

// Assist: convert_unit_struct_to_record_struct
//
// Converts a unit struct or enum variant into an empty record form and updates
// usages accordingly.
//
// ```
// struct Foo$0;
//
// impl Foo {
//     fn new() -> Self {
//         Foo
//     }
// }
// ```
// ->
// ```
// struct Foo {}
//
// impl Foo {
//     fn new() -> Self {
//         Foo {}
//     }
// }
// ```
pub(crate) fn convert_unit_struct_to_record_struct(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let strukt_or_variant = ctx
        .find_node_at_offset::<ast::Struct>()
        .map(Either::Left)
        .or_else(|| ctx.find_node_at_offset::<ast::Variant>().map(Either::Right))?;

    match &strukt_or_variant {
        Either::Left(strukt) => {
            let semicolon = strukt.semicolon_token()?;
            if strukt.field_list().is_some() {
                return None;
            }
            if ctx.offset() > semicolon.text_range().start() {
                return None;
            }
        }
        Either::Right(variant) => {
            if variant.field_list().is_some() || variant.expr().is_some() {
                return None;
            }
        }
    }

    let strukt_def = match &strukt_or_variant {
        Either::Left(s) => Either::Left(ctx.sema.to_def(s)?),
        Either::Right(v) => Either::Right(ctx.sema.to_def(v)?),
    };

    let target = strukt_or_variant.as_ref().either(|s| s.syntax(), |v| v.syntax()).text_range();

    acc.add(
        AssistId::refactor_rewrite("convert_unit_struct_to_record_struct"),
        "Convert to record struct",
        target,
        |edit| {
            edit_struct_references(ctx, edit, strukt_def);
            edit_struct_def(ctx, edit, &strukt_or_variant);
        },
    )
}

fn edit_struct_def(
    ctx: &AssistContext<'_>,
    edit: &mut SourceChangeBuilder,
    strukt: &Either<ast::Struct, ast::Variant>,
) {
    edit.edit_file(ctx.vfs_file_id());

    match strukt {
        Either::Left(strukt) => {
            let semicolon = match strukt.semicolon_token() {
                Some(it) => it,
                None => return,
            };

            let ws_text = semicolon
                .prev_token()
                .filter(|tok| tok.kind() == SyntaxKind::WHITESPACE)
                .map(|tok| {
                    let text = tok.text().to_owned();
                    edit.delete(tok.text_range());
                    text
                });

            let mut replacement = String::new();
            if let Some(ref text) = ws_text {
                replacement.push_str(text);
            } else if strukt.where_clause().is_some() {
                replacement.push('\n');
            } else {
                replacement.push(' ');
            }
            replacement.push_str("{}");
            edit.replace(semicolon.text_range(), replacement);
        }
        Either::Right(variant) => {
            let insert_at = variant.syntax().text_range().end();
            edit.insert(insert_at, " {}");
        }
    }
}

fn edit_struct_references(
    ctx: &AssistContext<'_>,
    edit: &mut SourceChangeBuilder,
    strukt: Either<hir::Struct, hir::Variant>,
) {
    let strukt_def = match strukt {
        Either::Left(s) => Definition::Adt(hir::Adt::Struct(s)),
        Either::Right(v) => Definition::Variant(v),
    };

    let usages = strukt_def.usages(&ctx.sema).include_self_refs().all();

    for (file_id, refs) in usages {
        edit.edit_file(file_id.file_id(ctx.db()));
        for r in &refs {
            process_reference(ctx, r, edit);
        }
    }
}

fn process_reference(
    ctx: &AssistContext<'_>,
    reference: &FileReference,
    edit: &mut SourceChangeBuilder,
) -> Option<()> {
    let name_like = reference.name.clone().into_name_like()?;
    match name_like {
        ast::NameLike::NameRef(name_ref) => {
            let full_path = name_ref.syntax().ancestors().find_map(ast::Path::cast)?;
            let segment_name = full_path.segment()?.name_ref()?;
            if segment_name.syntax().text_range() != name_ref.syntax().text_range() {
                return None;
            }
            let parent = full_path.syntax().parent()?;
            match_ast! {
                match parent {
                    ast::PathExpr(path_expr) => {
                        let file_range = ctx.sema.original_range_opt(path_expr.syntax())?;
                        let path_text = full_path.syntax().text().to_string();
                        edit.replace(file_range.range, format!("{path_text} {{}}"));
                    },
                    ast::PathPat(path_pat) => {
                        let file_range = ctx.sema.original_range_opt(path_pat.syntax())?;
                        let path = path_pat.path()?;
                        edit.replace(file_range.range, format!("{path} {{}}"));
                    },
                    _ => return None,
                }
            }
            Some(())
        }
        ast::NameLike::Name(name) => {
            let ident_pat = name.syntax().ancestors().find_map(ast::IdentPat::cast)?;
            let file_range = ctx.sema.original_range_opt(ident_pat.syntax())?;
            let path_text = name.text().to_string();
            edit.replace(file_range.range, format!("{path_text} {{}}"));
            Some(())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::convert_unit_struct_to_record_struct;

    #[test]
    fn not_applicable_tuple_struct() {
        check_assist_not_applicable(convert_unit_struct_to_record_struct, r#"struct Foo$0(u32);"#);
    }

    #[test]
    fn convert_unit_struct() {
        check_assist(
            convert_unit_struct_to_record_struct,
            r#"
struct Foo$0;

impl Foo {
    fn new() -> Foo {
        Foo
    }

    fn take(self) {
        match self {
            Foo => {}
        }
    }
}
"#,
            r#"
struct Foo {}

impl Foo {
    fn new() -> Foo {
        Foo {}
    }

    fn take(self) {
        match self {
            Foo {} => {}
        }
    }
}
"#,
        );
    }

    #[test]
    fn convert_unit_variant() {
        check_assist(
            convert_unit_struct_to_record_struct,
            r#"
enum E {
    $0Foo,
}

fn make() -> E {
    E::Foo
}
"#,
            r#"
enum E {
    Foo {},
}

fn make() -> E {
    E::Foo {}
}
"#,
        );
    }

    #[test]
    fn not_applicable_variant_with_discriminant() {
        check_assist_not_applicable(
            convert_unit_struct_to_record_struct,
            r#"
enum E {
    $0Foo = 1,
}
"#,
        );
    }
}
