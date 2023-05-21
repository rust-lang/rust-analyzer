use hir::{AssocItem, HasSource};
use std::iter;
use text_edit::TextRange;

use ide_db::RootDatabase;
use itertools::Itertools;
use syntax::{
    ast::{self, make, HasGenericParams, HasName},
    AstNode, SyntaxNode,
};

use crate::{
    utils::{convert_param_list_to_arg_list, generate_trait_impl_text_intransitive},
    AssistContext, AssistId, AssistKind, Assists,
};

// Assist: generate_trait_impl_for_enum
//
// Generate trait impl from enum where predicate.
//
// ```
// trait Trait {
//     type Input;
//     fn method(&self, input: Self::Input) -> Self;
// }
//
// impl Trait for u32 {
//     type Input = Self;
//     fn method(self: &Self, input: Self::Input) -> Self { Default::default() }
// }
//
// enum Enum where u32: $0Trait {
//     V1(i32, u32, i32),
//     V2 { name: u32, age: i32 },
//     V3 { age: u32, name: u32 },
//     v4 (String, u32)
//     v5()
//     v6{}
//     v7
// }
// ```
// ->
// ```
// trait Trait {
//     type Input;
//     fn method(&self, input: Self::Input) -> Self;
// }
//
// impl Trait for u32 {
//     type Input = Self;
//     fn method(self: &Self, input: Self::Input) -> Self { Default::default() }
// }
//
// enum Enum {
//     V1(i32, u32, i32),
//     V2 { name: u32, age: i32 },
//     V3 { age: u32, name: u32 },
//     v4 (String, u32)
//     v5()
//     v6{}
//     v7
// }
//
// impl Trait for Enum
// where u32: Trait
// {
//     type Input = u32;
//
//     fn method(self: &Self, input: Self::Input) -> Self {
//         match Self {
//             Self::V1 (f0, f1, f2) => Self::V1(f0, f1.method(input), f2),
//             Self::V2 { name, age } => Self::V2 { name: name.method(input), age },
//             Self::V3 { age, name } => Self::V3 { age: age.method(input), name },
//             Self::v4 (f0, f1) => Self::v4(f0, f1.method(input)),
//             variant => variant
//         }
//     }
// }
// ```
pub(crate) fn generate_trait_impl_for_enum(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    if !ctx.has_empty_selection() {
        return None;
    }

    // FIXME: Consume the where predicate on code action.

    // FIXME: Handle `Ty: Trait$0<..args> + Default` and `Ty: Default$0 + Trait<..args>` predicates.

    // FIXME: If Default is specified, and the method has a return type, each variant that doesn't
    // have a matching type can deligate:
    //   - all primitive/copy owned types to Default::default().
    //   - all primitive/copy non-mutable reference types to &Lit.
    //
    // Non-matching variants with these method return types are not supported:
    //   - Mutable references.
    //   - Non-const types.
    //     - They can be implemented with the once sync primitive, but that requires unsafe
    //       code blocks-so that's not an option.

    // FIXME: If there already exist an impl trait for the enum, halt/branch and repopulate.
    //   - Traits with generic params are allowed, as long as they don't collide with identical
    //     trait declarations.

    // FIXME: Doing a `where predicates` with Self projections and/or trait bound w/o Self projections
    // where `Trait` projector isn't implemented by Enum are not allowed.
    // - <Self as Trait>::Input: Trait<<Self as Trait>::Output, Self> { .. } - Not OK
    // - <u32  as Trait>::Input: Trait<<u32  as Trait>::Output, Self> { .. } - OK but not supported?

    // FIXME: Associated trait methods with `Self` typed params (not including receiver) is not supported.
    // - trait Trait { fn method(&self, param: Self) }

    // FIXME: Type bounds with assoc bindings are not handled correctly.

    // Check if we're inside an enum item. I think this is a quick check because we already
    // have a parsed AST we could check against, i.e. if offset is inside a Enum range, continue.
    let enum_at_offset = ctx.find_node_at_offset::<ast::Enum>()?;
    // We need to pick up the whole where predicate because of HRTBs.

    // FIXME: Support HRTBs bounds for impl. HRTBs can either be placed before the predicate
    // `for<'a> Type: Trait<'a>` or at bound position `Type: for<'a> Trait$0<'a>`, only the
    // latter would be matched as TypeBoundKind::ForType, unlike the former, that would be matched
    // as `TypeBoundKind::PathType`.
    let where_pred_at_offset = find_where_pred_without_hrtb(ctx)?;

    let ty_bound_path_at_offset = ctx.find_node_at_offset::<ast::PathType>()?;

    // Path takes you to the type definition
    let where_pred_path_ty =
        where_pred_at_offset.syntax().descendants().find_map(ast::PathType::cast)?;

    // FIXME: Handle trait const generics with default bounds
    let trait_hir = resolve_trait_path(&ctx.sema, &ty_bound_path_at_offset)?;
    let enum_hir = ctx.sema.to_def(&enum_at_offset)?;

    let generic_args_hir = get_trait_bound_generic_args(
        &ctx.sema,
        ty_bound_path_at_offset.syntax().descendants().find_map(ast::GenericArgList::cast).as_ref(),
    );

    let where_pred_ty_hir = resolve_ty_path(&ctx.sema, &where_pred_path_ty)?;

    let db = ctx.db();

    // Here we check if `Type<..>` in `Type<..>: Trait<..>` implements `Trait<..>`.
    //
    // Finding a trait solution that matches the obligation can result in either a unique or
    // ambiguous match. We are generally just interested in a unique match, but this doesn't tell us
    // anythig about that. I don't even know how chalk_ir works other than that it's comparable to
    // prolog.
    if !where_pred_ty_hir.impls_trait(db, trait_hir, &generic_args_hir) {
        return None;
    }

    // FIXME: This might not the the best impl finder. Given a `Trait<U, T> { .. }` and an `impl<U>
    // Trait<i32, U> for Type`, I'm not sure if this will do canonical matching.
    let trait_impl = hir::Impl::all_for_trait(db, trait_hir).into_iter().find_map(|imp| {
        (imp.self_ty(db).eq(&where_pred_ty_hir) && imp.trait_(db)?.eq(&trait_hir)).then_some(imp)
    })?;

    let where_pred_ty = where_pred_at_offset.ty()?;

    // FIXME: If no variants has any field that satisfies the where predicate, halt code action
    let variant_arms: Vec<(ast::Pat, ast::Path, hir::Variant)> = enum_hir
        .variants(db)
        .into_iter()
        .filter_map(|variant_hir| {
            create_partial_match_arm(
                &ctx.sema,
                &where_pred_ty,
                &variant_hir,
                &trait_hir,
                &generic_args_hir,
            )
        })
        .collect();

    let trait_impl_items = trait_impl.items(db);
    let assoc_items =
        create_body_impl(db, &trait_impl_items, &variant_arms, &where_pred_ty.to_string())?;

    let trait_text = ty_bound_path_at_offset.to_string();
    let adt = ast::Adt::Enum(enum_at_offset.clone());
    let code = assoc_items.iter().map(|assoc| format!("    {assoc}")).join("\n\n");

    // Trait impl is transitive, but we don't have to explicitly declare it in a where clause.
    let impl_trait_for_enum = generate_trait_impl_text_intransitive(&adt, &trait_text, &code);
    let enum_name = enum_at_offset.name()?;

    let id = AssistId("generate_trait_impl_for_enum", AssistKind::Generate);
    let label = format!("Generate `{trait_text}` impl for `{enum_name}`");
    let target = where_pred_at_offset.syntax().text_range();

    acc.add(id, label, target, |edit| {
        // Enum needs to have a where predicate to even get here, and if it has a where predicate,
        // it must have a where clause parent.
        let where_clause_syn = where_pred_at_offset.syntax().parent().unwrap();
        let where_pred_syn = where_pred_at_offset.syntax();

        // FIXME: Trailing comma w/ or w/o ws needs to be handled also.
        if where_clause_syn.children().count() == 1 {
            // enum Enum| |where Ty: Trait| {
            //           ^^^^^^^^^^^^^^^^^^
            //           Remove clause and whitespace
            //
            // This could probably be done with the `ted` module?
            let enum_end = where_clause_syn.prev_sibling().unwrap().text_range().end();
            edit.delete(TextRange::new(enum_end, where_clause_syn.text_range().end()))
        } else {
            where_clause_syn.children().for_each(|node| {
                if equal_in_kinds_and_tokens(&node, where_pred_syn) {
                    if let Some(sibling) = node.next_sibling() {
                        // There must exists a prev token if we match another sibling.
                        edit.delete(
                            node.text_range()
                                .cover(sibling.prev_sibling_or_token().unwrap().text_range()),
                        )
                    } else {
                        edit.delete(node.text_range())
                    }
                }
            });
        }

        let start_offset = enum_at_offset.syntax().text_range().end();

        match ctx.config.snippet_cap {
            Some(cap) => {
                edit.insert_snippet(cap, start_offset, impl_trait_for_enum);
            }
            None => {
                edit.insert(start_offset, impl_trait_for_enum);
            }
        }
    });

    Some(())
}

// --------------------- Create functions ----------------------------

fn create_body_impl(
    db: &RootDatabase,
    assoc_items: &[hir::AssocItem],
    variant_arms: &[(ast::Pat, ast::Path, hir::Variant)],
    where_pred_ty: &str,
) -> Option<Vec<ast::AssocItem>> {
    let mut impl_body = vec![];

    for assoc in assoc_items {
        match assoc {
            AssocItem::Function(method) => impl_body.push(ast::AssocItem::Fn(
                create_assoc_fn_impl(db, method, variant_arms, where_pred_ty)?,
            )),
            AssocItem::TypeAlias(ty_alias) => impl_body.push(ast::AssocItem::TypeAlias(
                create_assoc_ty_alias_impl(db, ty_alias, where_pred_ty)?,
            )),
            _ => (),
        }
    }

    Some(impl_body)
}

fn create_assoc_fn_impl(
    db: &RootDatabase,
    method: &hir::Function,
    variant_arms: &[(ast::Pat, ast::Path, hir::Variant)],
    _where_pred_ty: &str,
) -> Option<ast::Fn> {
    let method_source = method.source(db)?.value;

    // FIXME: Allow for async/unsafe-ness?
    if method.is_async(db) || method.is_unsafe_to_call(db) {
        return None;
    }

    let method_name = method_source.name()?.to_string();

    let method_generics =
        method_source.generic_param_list().as_ref().map(ToString::to_string).unwrap_or_default();

    let method_params_normalized = method_source.param_list()?.syntax().to_string();

    let method_ret_type = method_source.ret_type();

    let path_operator = if !method.has_self_param(db) { "::" } else { "." };
    let method_callable_args = convert_param_list_to_arg_list(method_source.param_list()?);

    // FIXME: We should resolve type aliases if for some reason there a Self::Output = Self binding.
    let method_ret_type_is_self = method_ret_type
        .as_ref()
        .and_then(|ret| ret.ty())
        .map(|ty| {
            let ty_str = ty.to_string();
            ty_str == "Self" || ty_str == "&Self"
        })
        .unwrap_or_default();

    let match_arm_pats = variant_arms
        .iter()
        .enumerate()
        .map(|(i, (pat, name, variant))| {
            let indent = if i == 0 { "" } else { "      " };

            if method_ret_type_is_self {
                let fields = get_fields(db, variant);
                match pat {
                    ast::Pat::RecordPat(rec) => {
                        let path = rec.path().unwrap();
                        let params = fields.replace(
                            &name.to_string(),
                            &format!(
                                "{name}: {name}{path_operator}{method_name}{method_callable_args}"
                            ),
                        );
                        format!("{indent}{path} {{ {fields} }} => {path} {{ {params} }}")
                    }
                    ast::Pat::TupleStructPat(tup) => {
                        let path = tup.path().unwrap();
                        let params = fields.replace(
                            &name.to_string(),
                            &format!("{name}{path_operator}{method_name}{method_callable_args}"),
                        );
                        format!("{indent}{path} ({fields}) => {path}({params})")
                    }
                    _ => unreachable!(
                        "Should not be possible to get here because of `create_partial_match_arm`"
                    ),
                }
            } else {
                format!("{indent}{pat} => {name}{path_operator}{method_name}{method_callable_args}")
            }
        })
        .join(",\n      ");

    let default_arm = if method_ret_type_is_self {
        "variant => variant".to_string()
    } else {
        format!("_ => {}", if method_ret_type.is_none() { "()" } else { "$0" })
    };

    let method_return = method_ret_type
        .as_ref()
        .map(ast::AstNode::syntax)
        .map(ToString::to_string)
        .unwrap_or_default();

    let method_where_clause =
        method_source.where_clause().map(|wc| wc.syntax().to_string()).unwrap_or_default();
    let method_body = format!(
        "{{\n        match Self {{\n            {match_arm_pats},\n            {default_arm}\n        }}\n    }}",
    );

    make::try_fn_(&format!(
        "fn {method_name}{method_generics}{method_params_normalized} {method_return} {method_where_clause}{method_body}"
    ))
}

fn create_assoc_ty_alias_impl(
    db: &RootDatabase,
    ty_alias: &hir::TypeAlias,
    where_pred_ty: &str,
) -> Option<ast::TypeAlias> {
    let source = ty_alias.source(db)?.value;
    let where_clause = source.where_clause();
    // FIXME: Make sure all projections works with resolved Self.

    // Replacing all Self keywords should be fine.
    let assignment = source
        .ty()
        .map(|ty| (make::ty(&ty.to_string().replace("Self", where_pred_ty)), where_clause));

    Some(make::ty_alias(
        ty_alias.name(db).as_str()?,
        source.generic_param_list(),
        None,
        None,
        assignment,
    ))
}

fn create_partial_match_arm(
    sema: &'_ hir::Semantics<'_, RootDatabase>,
    where_pred_ty: &ast::Type,
    variant_hir: &hir::Variant,
    trait_hir: &hir::Trait,
    generic_args: &[hir::Type],
) -> Option<(ast::Pat, ast::Path, hir::Variant)> {
    let variant_name_hir = variant_hir.name(sema.db);
    let variant_name = variant_name_hir.as_str()?;
    let variant_path = make::path_from_text(&format!("Self::{variant_name}"));
    let pred_ty_str = where_pred_ty.to_string();

    for field in variant_hir.fields(sema.db) {
        let field_ty = field.source(sema.db).and_then(get_type_from_field_source)?;

        if pred_ty_str != "Self" && field_ty.to_string() != pred_ty_str {
            continue;
        }

        let resolved_field_ty =
            resolve_ty_path(sema, &field_ty.syntax().descendants().find_map(ast::PathType::cast)?)?;

        // FIXME: This comparing is not reliable, i.e. it does not take into account canonical matches.
        // `enum<T> Enum where A<i32>: $0Trait { V(A<T>) }`
        if field_ty.to_string() == pred_ty_str
            || resolved_field_ty.impls_trait(sema.db, *trait_hir, generic_args)
        {
            let nameorindex = field.name(sema.db);
            match variant_hir.kind(sema.db) {
                hir::StructKind::Record => {
                    let field_path = make::path_unqualified(make::path_segment(make::name_ref(
                        nameorindex.as_str()?,
                    )));
                    let record_pat_field_list = itertools::chain(
                        iter::once(make::path_pat(field_path.clone())),
                        iter::once(ast::Pat::RestPat(make::rest_pat())),
                    );

                    let struct_pat =
                        ast::Pat::RecordPat(make::record_pat(variant_path, record_pat_field_list));

                    return Some((struct_pat, field_path, *variant_hir));
                }
                hir::StructKind::Tuple => {
                    let idx = nameorindex.as_tuple_index().unwrap();
                    let field_path = make::path_from_text(&format!("f{idx}"));

                    let mut pats = vec![];

                    for i in 0..=idx {
                        if i == idx {
                            pats.push(make::path_pat(field_path.clone()));
                            pats.push(ast::Pat::RestPat(make::rest_pat()));

                            break;
                        } else {
                            pats.push(ast::Pat::WildcardPat(make::wildcard_pat()))
                        }
                    }

                    let tuple_pat = ast::Pat::TupleStructPat(make::tuple_struct_pat(
                        variant_path,
                        pats.into_iter(),
                    ));

                    return Some((tuple_pat, field_path, *variant_hir));
                }
                hir::StructKind::Unit => unreachable!(
                    "Should not be possible to reach this? Unless ()::method_call counts?"
                ),
            }
        }
    }
    None
}

// --------------------- Helper functions ----------------------------

fn find_where_pred_without_hrtb(ctx: &AssistContext<'_>) -> Option<ast::WherePred> {
    let where_pred_at_offset = ctx.find_node_at_offset::<ast::WherePred>()?;

    // Make sure we check type bound at offset and not naively check after any `for` token.
    // e.g where Ty: for<'a> Trait<'a> + $0TraitMark should be valid because we are dispatching
    // `TraitMark`, not `Trait<'a>`
    //
    // Nested HRTBs are not supported `for<'a> Strukt<'a>: TraitA<'a> + for<'b> TraitB<'b> {}`
    if where_pred_at_offset.for_token().is_some()
        || ctx
            .find_node_at_offset::<ast::TypeBound>()?
            .syntax()
            .children()
            .find_map(ast::ForType::cast)
            .is_some()
    {
        return None;
    }

    Some(where_pred_at_offset)
}

fn equal_in_kinds_and_tokens(left: &SyntaxNode, right: &SyntaxNode) -> bool {
    // FIXME: There must be a better way to compare two node trees without comparing the text-range.
    // Note, this can fail if they diff in shape, even though they are equal in form.
    //
    // I think that this also fails when len(left) != len(right), iter = min(len(left), len(right))
    // because it can result in a false positive match?
    iter::zip(left.descendants_with_tokens(), right.descendants_with_tokens()).all(|(l, r)| {
        // .filter(|node| node.kind() != SyntaxKind::WHITESPACE);
        l.kind() == r.kind()
            && l.as_token()
                .and_then(|ll| r.as_token().map(|rr| ll.to_string() == rr.to_string()))
                .unwrap_or(true)
    })
}

fn resolve_ty_path(
    sema: &'_ hir::Semantics<'_, RootDatabase>,
    path: &ast::PathType,
) -> Option<hir::Type> {
    match sema.resolve_path(&path.path()?)? {
        hir::PathResolution::Def(def) => match def {
            hir::ModuleDef::BuiltinType(ty) => Some(ty.ty(sema.db)),
            hir::ModuleDef::Adt(ty) => Some(ty.ty(sema.db)),
            hir::ModuleDef::TypeAlias(ty) => Some(ty.ty(sema.db)),
            _ => None,
        },
        _ => None,
    }
}

fn resolve_trait_path(
    sema: &'_ hir::Semantics<'_, RootDatabase>,
    path: &ast::PathType,
) -> Option<hir::Trait> {
    match sema.resolve_path(&path.path()?)? {
        hir::PathResolution::Def(hir::ModuleDef::Trait(def)) => {
            // Unsafe traits should not be promoted as dispatchable.
            // Auto traits are not considered dispatchable.
            if def.is_unsafe(sema.db) || def.is_auto(sema.db) {
                return None;
            }
            Some(def)
        }
        _ => None,
    }
}

fn get_fields(db: &RootDatabase, variant: &hir::Variant) -> String {
    variant
        .fields(db)
        .into_iter()
        .map(|field| {
            let name = field.name(db);
            name.as_tuple_index()
                .map(|idx| format!("f{}", idx))
                .unwrap_or_else(|| name.as_str().unwrap().to_string())
        })
        .join(", ")
}

fn get_type_from_field_source(field: hir::InFile<hir::FieldSource>) -> Option<ast::Type> {
    match field.value {
        hir::FieldSource::Named(field) => field.ty(),
        hir::FieldSource::Pos(field) => field.ty(),
    }
}

fn get_trait_bound_generic_args(
    sema: &'_ hir::Semantics<'_, RootDatabase>,
    generic_args: Option<&ast::GenericArgList>,
) -> Vec<hir::Type> {
    generic_args
        .map(|list| {
            list.generic_args()
                .filter_map(|ga| {
                    ga.syntax()
                        .descendants()
                        .find_map(ast::Type::cast)
                        .and_then(|ty| sema.resolve_type(&ty))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    // ----------------------------------------------------------------------------
    #[test]
    fn test_wrong_trait_impl() {
        check_assist_not_applicable(
            generate_trait_impl_for_enum,
            r#"
trait Trait {
    type Input;
    fn method(&self, input: Self::Input) -> Self;
}

impl Trait for i32 {
    type Input = Self;
    fn method(&self, input: Self::Input) -> Self { Default::default() }
}

enum Enum where u32: $0Trait {
    One { inner: String, verycool: u32 },
    Two(i32, i32, u32)
}"#,
        );
    }

    // ----------------------------------------------------------------------------
    #[test]
    fn test_trait_impl_missing() {
        check_assist_not_applicable(
            generate_trait_impl_for_enum,
            r#"
trait Trait {
    type Input;
    fn method(&self, input: Self::Input) -> Self;
}

enum Enum where u32: $0Trait {
    One { inner: String, verycool: u32 },
    Two(i32, i32, u32)
}"#,
        );
    }

    // ----------------------------------------------------------------------------
    #[test]
    fn test_trait_missing() {
        check_assist_not_applicable(
            generate_trait_impl_for_enum,
            r#"
enum Enum where u32: $0Trait {
    One { inner: String, verycool: u32 },
    Two(i32, i32, u32)
}"#,
        );
    }

    // ----------------------------------------------------------------------------
    #[test]
    fn test_ty_not_exists() {
        check_assist_not_applicable(
            generate_trait_impl_for_enum,
            r#"
enum Enum where NoType: $0Trait {
    One { inner: String, verycool: u32 },
    Two(i32, i32, u32)
}"#,
        );
    }

    // ----------------------------------------------------------------------------
    #[test]
    fn test_generate_trait_impl_for_enum_random() {
        check_assist(
            generate_trait_impl_for_enum,
            r#"
trait Trait<T> {
    type Output;
    fn next(&self, input: T) -> Self::Output;
    fn next2();
}

impl Trait<u32> for u32 {
    type Output = String;
    fn next(&self, input: u32) -> Self::Output { Default::default() }
    fn next2() {}
}

impl Trait<i32> for i32 {
    type Output = String;
    fn next(&self, input: i32) -> Self::Output { Default::default() }
    fn next2() {}
}

enum A where u32: $0Trait<u32> { One { inner: String, verycool: u32 }, Two(i32, i32, u32) }"#,
            r#"
trait Trait<T> {
    type Output;
    fn next(&self, input: T) -> Self::Output;
    fn next2();
}

impl Trait<u32> for u32 {
    type Output = String;
    fn next(&self, input: u32) -> Self::Output { Default::default() }
    fn next2() {}
}

impl Trait<i32> for i32 {
    type Output = String;
    fn next(&self, input: i32) -> Self::Output { Default::default() }
    fn next2() {}
}

enum A { One { inner: String, verycool: u32 }, Two(i32, i32, u32) }

impl Trait<u32> for A
where u32: Trait<u32>
{
    type Output = String;

    fn next(&self, input: u32) -> Self::Output {
        match Self {
            Self::One { verycool, .. } => verycool.next(input),
            Self::Two(_, _, f2, ..) => f2.next(input),
            _ => $0
        }
    }

    fn next2()  {
        match Self {
            Self::One { verycool, .. } => verycool::next2(),
            Self::Two(_, _, f2, ..) => f2::next2(),
            _ => ()
        }
    }
}"#,
        );
    }

    // ----------------------------------------------------------------------------
    #[test]
    fn test_impl_trait_t_for_enum_worker() {
        check_assist(
            generate_trait_impl_for_enum,
            r#"
trait Trait {
    type Input;
    fn method(self, input: u32) -> Self;
}

impl Trait for u32 {
    type Input = Self;
    fn method(self, input: u32) -> Self {
        Default::default()
    }
}

enum Enum
where
    u32: Tr$0ait,
{
    V1(i32, u32, i32),
    V2 { name: u32, age: i32 },
    V3 { age: u32, name: u32 },
    v4(String, u32),
    v5(),
    v6 {},
    v7,
}
"#,
            r#"
trait Trait {
    type Input;
    fn method(self, input: u32) -> Self;
}

impl Trait for u32 {
    type Input = Self;
    fn method(self, input: u32) -> Self {
        Default::default()
    }
}

enum Enum
{
    V1(i32, u32, i32),
    V2 { name: u32, age: i32 },
    V3 { age: u32, name: u32 },
    v4(String, u32),
    v5(),
    v6 {},
    v7,
}

impl Trait for Enum
where
    u32: Trait,
{
    type Input = u32;

    fn method(self, input: u32) -> Self {
        match Self {
            Self::V1 (f0, f1, f2) => Self::V1(f0, f1.method(input), f2),
            Self::V2 { name, age } => Self::V2 { name: name.method(input), age },
            Self::V3 { age, name } => Self::V3 { age: age.method(input), name },
            Self::v4 (f0, f1) => Self::v4(f0, f1.method(input)),
            variant => variant
        }
    }
}
"#,
        );
    }
}
