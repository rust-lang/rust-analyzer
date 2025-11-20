use either::Either;
use ide_db::{defs::Definition, search::FileReference};
use syntax::{
    SyntaxKind,
    ast::{self, AstNode},
    match_ast,
};

use crate::{AssistContext, AssistId, Assists, assist_context::SourceChangeBuilder};

// Assist: convert_record_struct_to_unit_struct
//
// Converts an empty record struct or enum variant into a unit form and updates
// usages accordingly.
//
// ```
// struct Foo$0 {}
//
// impl Foo {
//     fn new() -> Self {
//         Foo {}
//     }
// }
// ```
// ->
// ```
// struct Foo;
//
// impl Foo {
//     fn new() -> Self {
//         Foo
//     }
// }
// ```
pub(crate) fn convert_record_struct_to_unit_struct(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let strukt_or_variant = ctx
        .find_node_at_offset::<ast::Struct>()
        .map(Either::Left)
        .or_else(|| ctx.find_node_at_offset::<ast::Variant>().map(Either::Right))?;

    let field_list = strukt_or_variant.as_ref().either(|s| s.field_list(), |v| v.field_list())?;
    let record_fields = match field_list {
        ast::FieldList::RecordFieldList(list) => list,
        _ => return None,
    };

    if record_fields.fields().next().is_some() {
        return None;
    }

    if ctx.offset() > record_fields.syntax().text_range().start() {
        return None;
    }

    let strukt_def = match &strukt_or_variant {
        Either::Left(s) => Either::Left(ctx.sema.to_def(s)?),
        Either::Right(v) => Either::Right(ctx.sema.to_def(v)?),
    };

    let target = strukt_or_variant.as_ref().either(|s| s.syntax(), |v| v.syntax()).text_range();

    acc.add(
        AssistId::refactor_rewrite("convert_record_struct_to_unit_struct"),
        "Convert to unit struct",
        target,
        |edit| {
            edit_struct_references(ctx, edit, strukt_def);
            edit_struct_def(ctx, edit, &strukt_or_variant, &record_fields);
        },
    )
}

fn edit_struct_def(
    ctx: &AssistContext<'_>,
    edit: &mut SourceChangeBuilder,
    strukt: &Either<ast::Struct, ast::Variant>,
    record_fields: &ast::RecordFieldList,
) {
    edit.edit_file(ctx.vfs_file_id());

    let newline_before = record_fields
        .l_curly_token()
        .and_then(|tok| tok.prev_token())
        .filter(|tok| tok.kind() == SyntaxKind::WHITESPACE)
        .map(|tok| {
            let has_newline = tok.text().contains('\n');
            edit.delete(tok.text_range());
            has_newline
        })
        .unwrap_or(false);

    match strukt {
        Either::Left(_) => {
            let replacement = if newline_before { "\n;" } else { ";" };
            edit.replace(record_fields.syntax().text_range(), replacement);
        }
        Either::Right(_) => {
            edit.delete(record_fields.syntax().text_range());
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
        for r in refs {
            process_reference(ctx, r, edit);
        }
    }
}

fn process_reference(
    ctx: &AssistContext<'_>,
    reference: FileReference,
    edit: &mut SourceChangeBuilder,
) -> Option<()> {
    let name_ref = reference.name.as_name_ref()?;
    let path_segment = name_ref.syntax().parent().and_then(ast::PathSegment::cast)?;
    let full_path =
        path_segment.syntax().parent()?.ancestors().map_while(ast::Path::cast).last()?;

    if full_path.segment()?.name_ref()? != *name_ref {
        return None;
    }

    let parent = full_path.syntax().parent()?;
    match_ast! {
        match parent {
            ast::RecordExpr(record_expr) => {
                let file_range = ctx.sema.original_range_opt(record_expr.syntax())?;
                let path = record_expr.path()?;
                edit.replace(file_range.range, path.syntax().text().to_string());
            },
            ast::RecordPat(record_pat) => {
                let file_range = ctx.sema.original_range_opt(record_pat.syntax())?;
                let path = record_pat.path()?;
                edit.replace(file_range.range, path.syntax().text().to_string());
            },
            _ => return None,
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::convert_record_struct_to_unit_struct;

    #[test]
    fn not_applicable_non_empty_record_struct() {
        check_assist_not_applicable(
            convert_record_struct_to_unit_struct,
            r#"struct Foo$0 { field: u32 }"#,
        );
    }

    #[test]
    fn convert_empty_struct() {
        check_assist(
            convert_record_struct_to_unit_struct,
            r#"
struct Foo$0 {}

impl Foo {
    fn new() -> Self {
        Foo {}
    }

    fn take(self) {
        let Foo {} = self;
    }
}
"#,
            r#"
struct Foo;

impl Foo {
    fn new() -> Self {
        Foo
    }

    fn take(self) {
        let Foo = self;
    }
}
"#,
        );
    }

    #[test]
    fn convert_record_variant() {
        check_assist(
            convert_record_struct_to_unit_struct,
            r#"
enum E {
    $0Foo {}
}

fn make() -> E {
    E::Foo {}
}
"#,
            r#"
enum E {
    Foo
}

fn make() -> E {
    E::Foo
}
"#,
        );
    }
}
