use ide_db::{
    assists::{AssistId, AssistKind},
    helpers::mod_path_to_ast,
};
use syntax::{ast, AstNode, TextRange};

use crate::assist_context::{AssistContext, Assists, SourceChangeBuilder};

// Assist: destructure_struct_binding
//
// Destructure a struct binding in place.
//
// ```
// struct Struct { a: u8, b: u8 }
//
// fn main() {
//     let $0x = Struct { a: 1, b: 2 };
// }
// ```
// ->
// ```
// struct Struct { a: u8, b: u8 }
//
// fn main() {
//     let Struct { a: $0_a, b: _b } = Struct { a: 1, b: 2 };
// }
// ```
pub(crate) fn destructure_struct_binding(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let ident_pat: ast::IdentPat = ctx.find_node_at_offset()?;
    let data = collect_data(ident_pat, ctx)?;

    acc.add(
        AssistId("destructure_struct_binding", AssistKind::RefactorRewrite),
        "Destructure struct binding",
        data.range,
        |builder| {
            edit_struct_assignment(ctx, builder, &data);
            // FIXME: add struct usages.
        },
    );
    Some(())
}

struct StructData {
    ident_pat: ast::IdentPat,
    range: TextRange,
    struct_kind: hir::StructKind,
    struct_def_path: hir::ModPath,
    fields: Vec<hir::Field>,
}

fn collect_data(ident_pat: ast::IdentPat, ctx: &AssistContext<'_>) -> Option<StructData> {
    let ty = ctx.sema.type_of_pat(&ident_pat.clone().into())?.adjusted();

    // The type might be a reference.
    let ty = ty.strip_references();

    // The type is required to be a struct.
    let strukt = match ty.as_adt()? {
        hir::Adt::Struct(strukt) => strukt,
        _ => return None,
    };

    let module = ctx.sema.scope(ident_pat.syntax())?.module();
    let struct_def = hir::ModuleDef::from(strukt);
    let struct_kind = strukt.kind(ctx.db());
    let struct_def_path = module.find_use_path(ctx.db(), struct_def, ctx.config.prefer_no_std)?;

    let range = ident_pat.syntax().text_range();

    let fields = strukt.fields(ctx.db());

    Some(StructData { ident_pat, range, struct_kind, struct_def_path, fields })
}

fn edit_struct_assignment(
    ctx: &AssistContext<'_>,
    builder: &mut SourceChangeBuilder,
    data: &StructData,
) {
    let struct_path = mod_path_to_ast(&data.struct_def_path);
    let is_ref = data.ident_pat.ref_token().is_some();
    let is_mut = data.ident_pat.mut_token().is_some();

    let field_names = names_of_fields(ctx, data.struct_kind, &data.fields);
    let ident_pats = field_names.iter().map(|field_name| {
        let name = ast::make::name(field_name);
        ast::Pat::from(ast::make::ident_pat(is_ref, is_mut, name))
    });
    let struct_pat: ast::Pat = match data.struct_kind {
        hir::StructKind::Tuple => {
            ast::Pat::TupleStructPat(ast::make::tuple_struct_pat(struct_path, ident_pats))
        }
        hir::StructKind::Record => {
            let fields = ident_pats.zip(&data.fields).map(|(pat, field)| {
                let field_name = field.name(ctx.db()).to_smol_str();
                ast::make::record_pat_field(ast::make::name_ref(&field_name), pat)
            });
            let field_list = ast::make::record_pat_field_list(fields);
            ast::Pat::RecordPat(ast::make::record_pat_with_fields(struct_path, field_list))
        }
        // Bare identifier pattern, which matches the unit struct that it names.
        hir::StructKind::Unit => ast::make::path_pat(struct_path),
    };

    let text = struct_pat.to_string();
    match ctx.config.snippet_cap {
        Some(cap) => {
            let snip = {
                // place cursor on first item
                match names_of_fields(ctx, data.struct_kind, &data.fields).get(0) {
                    Some(first_field) => text.replacen(first_field, &format!("$0{first_field}"), 1),
                    None => format!("$0{text}"),
                }
            };
            builder.replace_snippet(cap, data.range, snip);
        }
        None => builder.replace(data.range, text),
    };
}

fn names_of_fields(
    ctx: &AssistContext<'_>,
    struct_kind: hir::StructKind,
    fields: &Vec<hir::Field>,
) -> Vec<String> {
    match struct_kind {
        hir::StructKind::Tuple => {
            (0..fields.len())
                .map(|index| {
                    // FIXME: detect if generated name already used
                    format!("_{}", index)
                })
                .collect()
        }
        hir::StructKind::Record => {
            fields
                .iter()
                .map(|field| {
                    let field_name = field.name(ctx.db()).to_smol_str();
                    // FIXME: detect if generated name already used
                    format!("_{}", field_name)
                })
                .collect()
        }
        hir::StructKind::Unit => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::{check_assist, check_assist_not_applicable};
    #[test]
    fn dont_trigger_on_number() {
        check_assist_not_applicable(
            destructure_struct_binding,
            r#"
fn main() {
let $0v = 42;
}
            "#,
        )
    }
    #[test]
    fn dont_trigger_on_tuple() {
        check_assist_not_applicable(
            destructure_struct_binding,
            r#"
fn main() {
    let $0v = (1, 2);
}
            "#,
        )
    }

    #[test]
    fn destructure_3_tuple_struct() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Foo(u8, u8, u8);

fn main() {
    let $0st = Foo(1,2,3);
}
            "#,
            r#"
struct Foo(u8, u8, u8);

fn main() {
    let Foo($0_0, _1, _2) = Foo(1,2,3);
}
            "#,
        )
    }

    #[test]
    fn destructure_record_struct() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Point {x: i32, y: i32};

fn main() {
    let $0pt = Point {x: 1, y: -1};
}
            "#,
            r#"
struct Point {x: i32, y: i32};

fn main() {
    let Point { x: $0_x, y: _y } = Point {x: 1, y: -1};
}
            "#,
        )
    }
    #[test]
    fn destructure_unit_struct() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Marker;

fn main() {
    let $0mk = Marker;
}
            "#,
            r#"
struct Marker;

fn main() {
    let $0Marker = Marker;
}
            "#,
        )
    }
    #[test]
    fn destructure_record_struct_in_other_module() {
        check_assist(
            destructure_struct_binding,
            r#"
mod other {
    pub struct Wrapper {pub(super) inner: bool};
}

fn main() {
    let $0wrapped = other::Wrapper{ inner: false };
}
            "#,
            r#"
mod other {
    pub struct Wrapper {pub(super) inner: bool};
}

fn main() {
    let other::Wrapper { inner: $0_inner } = other::Wrapper{ inner: false };
}
            "#,
        )
    }
    #[test]
    fn destructure_in_parameter() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Foo { bar: (), baz: () };

fn func($0foo: Foo) {}
            "#,
            r#"
struct Foo { bar: (), baz: () };

fn func(Foo { bar: $0_bar, baz: _baz }: Foo) {}
            "#,
        )
    }
    #[test]
    fn destructure_ref() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Foo { bar: () };

fn main() {
    let &$0foo = &Foo { bar: () };
}
            "#,
            r#"
struct Foo { bar: () };

fn main() {
    let &Foo { bar: $0_bar } = &Foo { bar: () };
}
            "#,
        )
    }

    #[test]
    fn destructure_inner_ref() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Foo { bar: () };

fn main() {
    let $0foo = &Foo { bar: () };
}
            "#,
            r#"
struct Foo { bar: () };

fn main() {
    let Foo { bar: $0_bar } = &Foo { bar: () };
}
            "#,
        )
    }
    #[test]
    fn with_mut() {
        check_assist(
            destructure_struct_binding,
            r#"
struct Nums(i16, u8);

fn main() {
    let mut $0t = Nums(1,2);
}
            "#,
            r#"
struct Nums(i16, u8);

fn main() {
    let Nums(mut $0_0, mut _1) = Nums(1,2);
}
            "#,
        )
    }
}
