use ide_db::{FxIndexSet, syntax_helpers::suggest_name::NameGenerator};
use syntax::{
    NodeOrToken, SmolStr, T,
    ast::{
        self, AstNode, HasGenericParams, HasName,
        make::{self, tokens},
        syntax_factory::SyntaxFactory,
    },
    syntax_editor::{Position, SyntaxAnnotation, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: add_missing_lifetime
//
// Adds missing lifetimes to a struct, enum or union.
//
// ```
// struct $0Foo<T> {
//     x: &'a i32,
//     y: &T
// }
// ```
// ->
// ```
// struct Foo<'a, ${0:'b}, T> {
//     x: &'a i32,
//     y: &${0:'b} T
// }
// ```

pub(crate) fn add_missing_lifetime(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let node = ctx.find_node_at_offset::<ast::Adt>()?;
    let all_inner_refs = fetch_all_refs(&node)?;
    let (refs_without_lifetime, refs_with_lifetime): (Vec<_>, Vec<_>) =
        all_inner_refs.into_iter().partition(|ref_type| ref_type.lifetime().is_none());

    let adt_declared_lifetimes: FxIndexSet<SmolStr> = node
        .generic_param_list()
        .map(|gen_list| {
            gen_list
                .lifetime_params()
                .filter_map(|lt| lt.lifetime())
                .map(|lt| lt.text().into())
                .collect()
        })
        .unwrap_or_default();

    let adt_undeclared_lifetimes: FxIndexSet<SmolStr> = refs_with_lifetime
        .iter()
        .filter_map(|ref_type| ref_type.lifetime())
        .map(|lt| lt.text().into())
        .filter(|lt_text| !adt_declared_lifetimes.contains(lt_text))
        .collect();

    if refs_without_lifetime.is_empty() && adt_undeclared_lifetimes.is_empty() {
        return None;
    }

    let all_existing_lifetimes: Vec<SmolStr> =
        adt_declared_lifetimes.iter().chain(adt_undeclared_lifetimes.iter()).cloned().collect();

    add_and_declare_lifetimes(
        acc,
        ctx,
        &node,
        adt_undeclared_lifetimes,
        refs_without_lifetime,
        all_existing_lifetimes,
    )
}

fn add_and_declare_lifetimes(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
    node: &ast::Adt,
    adt_undeclared_lifetimes: FxIndexSet<SmolStr>,
    refs_without_lifetime: Vec<ast::RefType>,
    all_existing_lifetimes: Vec<SmolStr>,
) -> Option<()> {
    let has_refs_without_lifetime = !refs_without_lifetime.is_empty();
    let has_undeclared_lifetimes = !adt_undeclared_lifetimes.is_empty();

    let message = match (has_refs_without_lifetime, has_undeclared_lifetimes) {
        (false, true) => "Declare used lifetimes in generic parameters",
        (true, false) | (true, true) => "Add missing lifetimes",
        _ => return None,
    };

    let mut name_gen =
        NameGenerator::new_with_names(all_existing_lifetimes.iter().map(|s| s.as_str()));
    let new_lifetime_name =
        if has_refs_without_lifetime { name_gen.for_lifetime() } else { SmolStr::default() };

    acc.add(
        AssistId::quick_fix("add_missing_lifetime"),
        message,
        node.syntax().text_range(),
        |builder| {
            let make = SyntaxFactory::with_mappings();
            let mut editor = builder.make_editor(node.syntax());
            let comma_and_space = [make::token(T![,]).into(), tokens::single_space().into()];

            let mut lifetime_elements = vec![];
            let mut new_lifetime_to_annotate = None;

            if has_undeclared_lifetimes {
                for (i, lifetime_text) in adt_undeclared_lifetimes.iter().enumerate() {
                    (i > 0).then(|| lifetime_elements.extend(comma_and_space.clone()));
                    let new_lifetime = make.lifetime(lifetime_text);
                    lifetime_elements.push(new_lifetime.syntax().clone().into());
                }
            }

            if has_refs_without_lifetime {
                has_undeclared_lifetimes.then(|| lifetime_elements.extend(comma_and_space.clone()));
                let lifetime = make.lifetime(&new_lifetime_name);
                new_lifetime_to_annotate = Some(lifetime.clone());
                lifetime_elements.push(lifetime.syntax().clone().into());
            }

            if let Some(gen_param) = node.generic_param_list()
                && let Some(left_angle) = gen_param.l_angle_token()
            {
                if !lifetime_elements.is_empty() {
                    lifetime_elements.push(make::token(T![,]).into());
                    lifetime_elements.push(tokens::single_space().into());
                }
                editor.insert_all(Position::after(&left_angle), lifetime_elements);
            } else if let Some(name) = node.name()
                && !lifetime_elements.is_empty()
            {
                let mut final_elements = vec![make::token(T![<]).into()];
                final_elements.append(&mut lifetime_elements);
                final_elements.push(make::token(T![>]).into());
                editor.insert_all(Position::after(name.syntax()), final_elements);
            }

            let snippet = ctx.config.snippet_cap.map(|cap| builder.make_placeholder_snippet(cap));

            if let Some(lifetime) = new_lifetime_to_annotate
                && let Some(snippet) = snippet
            {
                editor.add_annotation(lifetime.syntax(), snippet);
            }

            if has_refs_without_lifetime {
                add_lifetime_to_refs(
                    refs_without_lifetime,
                    &new_lifetime_name,
                    &mut editor,
                    &make,
                    snippet,
                );
            }

            editor.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), editor);
            has_refs_without_lifetime.then(|| builder.rename());
        },
    )
}

fn fetch_all_refs(node: &ast::Adt) -> Option<Vec<ast::RefType>> {
    let ref_types: Vec<ast::RefType> = match node {
        ast::Adt::Enum(enum_) => enum_
            .variant_list()?
            .variants()
            .filter_map(|variant| find_all_ref_types_from_field_list(&variant.field_list()?))
            .flatten()
            .collect(),
        ast::Adt::Struct(strukt) => find_all_ref_types_from_field_list(&strukt.field_list()?)?,
        ast::Adt::Union(union) => union
            .record_field_list()?
            .fields()
            .filter_map(|r_field| {
                let ast::Type::RefType(ref_type) = r_field.ty()? else { return None };
                Some(ref_type)
            })
            .collect(),
    };

    (!ref_types.is_empty()).then_some(ref_types)
}

fn find_all_ref_types_from_field_list(field_list: &ast::FieldList) -> Option<Vec<ast::RefType>> {
    let ref_types: Vec<ast::RefType> = match field_list {
        ast::FieldList::RecordFieldList(record_list) => record_list
            .fields()
            .filter_map(|f| {
                let ast::Type::RefType(ref_type) = f.ty()? else { return None };
                Some(ref_type)
            })
            .collect(),
        ast::FieldList::TupleFieldList(tuple_field_list) => tuple_field_list
            .fields()
            .filter_map(|f| {
                let ast::Type::RefType(ref_type) = f.ty()? else { return None };
                Some(ref_type)
            })
            .collect(),
    };

    (!ref_types.is_empty()).then_some(ref_types)
}

fn add_lifetime_to_refs(
    refs_without_lifetime: Vec<ast::RefType>,
    lifetime_text: &str,
    editor: &mut SyntaxEditor,
    make: &SyntaxFactory,
    snippet: Option<SyntaxAnnotation>,
) {
    for r#ref in refs_without_lifetime {
        let Some(amp_token) = r#ref.amp_token() else { continue };
        let lifetime = make.lifetime(lifetime_text);
        let node_or_token = &NodeOrToken::Token(amp_token);
        let elements = vec![lifetime.syntax().clone().into(), tokens::single_space().into()];
        editor.insert_all(Position::after(node_or_token), elements);
        if let Some(snippet) = snippet {
            editor.add_annotation(lifetime.syntax(), snippet);
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn struct_lifetime() {
        check_assist(
            add_missing_lifetime,
            r#"
struct $0Foo {
    x: &'a i32,
    y: &'b u32
}"#,
            r#"
struct Foo<'a, 'b> {
    x: &'a i32,
    y: &'b u32
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
struct $0Foo<T> {
    x: &'a T
}"#,
            r#"
struct Foo<'a, T> {
    x: &'a T
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
struct Foo<T> {
    a: &$0T,
    b: usize
}"#,
            r#"
struct Foo<${0:'a}, T> {
    a: &${0:'a} T,
    b: usize
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
struct Foo<'a> {
    x: &'a i32,
    y: &$0u32
}"#,
            r#"
struct Foo<${0:'b}, 'a> {
    x: &'a i32,
    y: &${0:'b} u32
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
struct $0Foo {
    x: &'a i32,
    y: &u32
}"#,
            r#"
struct Foo<'a, ${0:'b}> {
    x: &'a i32,
    y: &${0:'b} u32
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
struct $0Foo<T> {
    x: &'a i32,
    y: &T
}"#,
            r#"
struct Foo<'a, ${0:'b}, T> {
    x: &'a i32,
    y: &${0:'b} T
}"#,
        );
    }

    #[test]
    fn enum_lifetime() {
        check_assist(
            add_missing_lifetime,
            r#"
enum $0Foo {
    Bar { x: &'a i32 }
}"#,
            r#"
enum Foo<'a> {
    Bar { x: &'a i32 }
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
enum $0Foo<T> {
    Bar {
        x: &'a i32,
        y: &'b T
    }
}"#,
            r#"
enum Foo<'a, 'b, T> {
    Bar {
        x: &'a i32,
        y: &'b T
    }
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
enum Foo {
    Bar { a: &$0i32 }
}"#,
            r#"
enum Foo<${0:'a}> {
    Bar { a: &${0:'a} i32 }
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
enum Foo<T> {
    Bar { a: T },
    Other,
    Tuple(u32, &$0u32)
}"#,
            r#"
enum Foo<${0:'a}, T> {
    Bar { a: T },
    Other,
    Tuple(u32, &${0:'a} u32)
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
enum Foo<T> {
    Bar {
        a: &$0i32,
        b: &T
    }
}"#,
            r#"
enum Foo<${0:'a}, T> {
    Bar {
        a: &${0:'a} i32,
        b: &${0:'a} T
    }
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
enum Foo<'a> {
    Bar {
        x: &'a i32,
        y: &$0u32
    }
}"#,
            r#"
enum Foo<${0:'b}, 'a> {
    Bar {
        x: &'a i32,
        y: &${0:'b} u32
    }
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
enum $0Foo {
    Bar { x: &'a i32 },
    Baz(&u32)
}"#,
            r#"
enum Foo<'a, ${0:'b}> {
    Bar { x: &'a i32 },
    Baz(&${0:'b} u32)
}"#,
        );
    }

    #[test]
    fn union_lifetime() {
        check_assist(
            add_missing_lifetime,
            r#"
union $0Foo {
    x: &'a i32
}"#,
            r#"
union Foo<'a> {
    x: &'a i32
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
union $0Foo<T> {
    x: &'a T
}"#,
            r#"
union Foo<'a, T> {
    x: &'a T
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
union Foo {
    a: &$0i32,
    b: &usize
}"#,
            r#"
union Foo<${0:'a}> {
    a: &${0:'a} i32,
    b: &${0:'a} usize
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
union Foo<T> {
    a: &$0T,
    b: usize
}"#,
            r#"
union Foo<${0:'a}, T> {
    a: &${0:'a} T,
    b: usize
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
union Foo<'a> {
    x: &'a i32,
    y: &$0u32
}"#,
            r#"
union Foo<${0:'b}, 'a> {
    x: &'a i32,
    y: &${0:'b} u32
}"#,
        );
        check_assist(
            add_missing_lifetime,
            r#"
union $0Foo<T> {
    x: &'a T,
    y: &i32
}"#,
            r#"
union Foo<'a, ${0:'b}, T> {
    x: &'a T,
    y: &${0:'b} i32
}"#,
        );
    }

    #[test]
    fn not_applicable_when_all_correct() {
        check_assist_not_applicable(add_missing_lifetime, r#"struct $0Foo<'a> { x: &'a i32 }"#);
        check_assist_not_applicable(
            add_missing_lifetime,
            r#"struct $0Foo<'a, 'b> { x: &'a i32, y: &'b u32 }"#,
        );
        check_assist_not_applicable(
            add_missing_lifetime,
            r#"enum $0Foo<'a> { Bar { x: &'a i32 } }"#,
        );
        check_assist_not_applicable(add_missing_lifetime, r#"enum Foo { Bar, $0Misc }"#);
        check_assist_not_applicable(add_missing_lifetime, r#"union Foo<'a> { a: &'a $0i32 }"#);
    }
}
