use hir::{AssocItem, HasSource};
use std::iter;

use text_edit::{TextRange, TextSize};

use ide_db::RootDatabase;
use itertools::Itertools;
use syntax::{
    ast::{self, make, HasGenericParams, HasName},
    AstNode, Direction, SyntaxKind, SyntaxNode,
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
//     fn method(self, input: Self::Input) -> Self;
// }
//
// impl Trait for u32 {
//     type Input = Self;
//     fn method(self, input: Self::Input) -> Self { Default::default() }
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
//     fn method(self, input: Self::Input) -> Self;
// }
//
// impl Trait for u32 {
//     type Input = Self;
//     fn method(self, input: Self::Input) -> Self { Default::default() }
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
//     fn method(self, input: Self::Input) -> Self {
//         match self {
//             Self::V1 (f0, f1, f2) => Self::V1(f0, f1.method(input), f2),
//             Self::V2 { name, age } => Self::V2 { name: name.method(input), age },
//             Self::V3 { age, name } => Self::V3 { age: age.method(input), name },
//             Self::v4 (f0, f1) => Self::v4(f0, f1.method(input)),
//             variant => variant$0
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
    let db = ctx.db();

    // There are two different ways trigger this code assist:
    // 1. Using a regular `Type: Trait`, such that the clause upholds, and there exists at least
    //    one variant with a matching field. Note that we always take the first field that
    //    matches the our type.
    //
    // |Currently not supported|
    // 2. Using a `Self: Trait` doesn't require Self to implement Trait, but there would still
    //    have to be at least one field that implements Trait for it to work. Note, because
    //    we've not specified a concrete type, we will check each field individually until we've
    //    found a candidate.
    //
    // If Default is specified, and the method has a return type, each variant that doesn't
    // have a matching type can deligate:
    // 1. all primitive/copy owned types to Default::default().
    // 2. all primitive/copy non-mutable reference types to &Lit.
    //
    // Non-matching variants with these method return types are not supported:
    // 1. Mutable references.
    // 2. Non-const types.
    //   a. They can be implemented with the once sync primitive, but that requires unsafe
    //      code blocks-so that's not an option.

    // FIXME: Handle `Ty: Trait$0<..args> + Default` and `Ty: Default$0 + Trait<..args>` predicates.

    // FIXME: If there already exist an impl trait for the enum, halt/branch and repopulate.
    // 1. Traits with generic params are allowed, as long as they don't collide with identical
    //    trait declarations.

    // FIXME: Doing a `where predicates` with Self projections and/or trait bound w/o Self projections
    // where `Trait` projector isn't implemented by Enum are not allowed.
    // 1. <Self as Trait>::Input: Trait<<Self as Trait>::Output, Self> { .. } - Not OK
    // 2. <u32  as Trait>::Input: Trait<<u32  as Trait>::Output, Self> { .. } - OK but not supported?

    // FIXME: Associated trait methods with `Self` typed params (not including receiver) is not supported.
    // 1. trait Trait { fn method(&self, param: Self) }

    // FIXME: Type bounds with assoc bindings are not handled correctly.

    // FIXME: Support HRTBs bounds for impl. HRTBs can either be placed before the predicate
    // `for<'a> Type: Trait<'a>` or at bound position `Type: for<'a> Trait$0<'a>`, only the
    // latter would be matched as TypeBoundKind::ForType, unlike the former, that would be matched
    // as `TypeBoundKind::PathType`.

    // FIXME: Handle trait const generics with default bounds

    // Check if we're inside an enum item. I think this is a quick check because we already
    // have a parsed AST we could check against, i.e. if offset is inside a Enum range, continue.
    let enum_at_offset = ctx.find_node_at_offset::<ast::Enum>()?;
    // We need to pick up the whole where predicate because of HRTBs.

    let where_pred_at_offset = find_where_pred_without_hrtb(ctx)?;
    let ty_bound_path_at_offset = ctx.find_node_at_offset::<ast::PathType>()?;

    // The `Type` in `Type: Trait<..args>`
    let where_pred_ty_hir = resolve_path_type(
        &ctx.sema,
        &where_pred_at_offset.syntax().descendants().find_map(ast::PathType::cast)?,
    )?;

    // The `Trait` in `Type: Trait<..args>`
    let trait_hir = resolve_trait_path(&ctx.sema, &ty_bound_path_at_offset)?;

    // The `..args` in `Type: Trait<..args>`
    let generic_args_hir = get_trait_bound_generic_args(
        &ctx.sema,
        ty_bound_path_at_offset.syntax().descendants().find_map(ast::GenericArgList::cast).as_ref(),
    );

    // Finding a trait solution that matches the obligation can result in either a unique or
    // ambiguous match. We are generally just interested in a unique match, but this doesn't tell us
    // anythig about that. I don't even know how chalk_ir works other than that it's comparable to
    // prolog.
    // (!where_pred_ty_hir.impls_trait(db, trait_hir, &generic_args_hir)).then_some(())?;

    // FIXME: This might not the the best impl finder. Given a `Trait<U, T> { .. }` and an `impl<U>
    // Trait<i32, U> for Type`, it won't do canonical matching. Also, imagine a trait being Clone,
    // or a trait with a blanket implementation, those could make this call blow up quickly.
    let trait_impl = hir::Impl::all_for_trait(db, trait_hir).find_map(|impl_| {
        // It seems better to resolve with path type
        // let ty = resolve_path_type(&ctx.sema, &impl_source.self_path_ty()?)?;
        (impl_.self_ty(db).eq(&where_pred_ty_hir) && impl_.trait_(db)?.eq(&trait_hir))
            .then_some(impl_)
    })?;

    let where_pred_ty_str = where_pred_at_offset.ty()?.to_string();
    let enum_hir = ctx.sema.to_def(&enum_at_offset)?;

    let variant_arms = enum_hir
        .variants(db)
        .into_iter()
        .filter_map(|variant_hir| {
            create_partial_match_arm(
                &ctx.sema,
                &where_pred_ty_str,
                &variant_hir,
                &trait_hir,
                &generic_args_hir,
            )
        })
        .collect_vec();

    // Make sure there's atleast one delegable variant.
    // Is there a style convention against these?
    variant_arms.first()?;

    let enum_name = enum_at_offset.name()?;
    let trait_text = ty_bound_path_at_offset.to_string();

    let code = create_impl_body(db, &trait_impl.items(db), &variant_arms, &where_pred_ty_str)?;
    let where_pred_syn = where_pred_at_offset.syntax();

    let id = AssistId("generate_trait_impl_for_enum", AssistKind::Generate);
    let label = format!("Generate `{trait_text}` impl for `{enum_name}`");
    let target = where_pred_syn.text_range();

    acc.add(id, label, target, |edit| {
        // Enum needs to have a where predicate to even get here, and if it has a where predicate,
        // it must have a where clause parent.
        let where_clause_syn = where_pred_syn.parent().unwrap();

        // FIXME: Trailing comma w/ or w/o ws needs to be handled also.
        if where_clause_syn.children().count() == 1 {
            // enum Enum| |where Ty: Trait| {
            //           ^^^^^^^^^^^^^^^^^^
            //           Remove clause and whitespace
            //
            // This could probably be done with the `ted` module?
            let start = get_sibling_token_text_size(&where_clause_syn, Direction::Prev);
            let end = get_sibling_token_text_size(&where_clause_syn, Direction::Next);

            edit.replace(TextRange::new(start, end), " ");
        } else {
            let mut siblings = where_pred_syn.siblings_with_tokens(syntax::Direction::Next);

            let start = get_sibling_token_text_size(where_pred_syn, Direction::Prev);
            let end = siblings
                .find_map(|sib| (sib.kind() == SyntaxKind::COMMA).then_some(sib.text_range().end()))
                .unwrap_or_else(|| where_pred_syn.text_range().end());

            edit.delete(TextRange::new(start, end))
        }

        let adt = ast::Adt::Enum(enum_at_offset);

        // Trait impl is transitive, but we don't have to explicitly declare it in a where clause.
        let impl_trait_for_enum = generate_trait_impl_text_intransitive(&adt, &trait_text, &code);
        let start_offset = adt.syntax().text_range().end();

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

fn create_impl_body(
    db: &RootDatabase,
    assoc_items: &[hir::AssocItem],
    variant_arms: &[(ast::Pat, ast::Path, hir::Variant)],
    where_pred_ty: &str,
) -> Option<String> {
    let mut impl_body = vec![];

    for assoc in assoc_items {
        match assoc {
            AssocItem::TypeAlias(ty_alias) => {
                impl_body.push(create_impl_assoc_ty_alias(db, ty_alias, where_pred_ty)?)
            }
            AssocItem::Function(method) => {
                impl_body.push(create_impl_assoc_method(db, method, variant_arms, where_pred_ty)?)
            }
            _ => (),
        }
    }

    Some(impl_body.iter().map(|assoc| format!("    {assoc}")).join("\n\n"))
}

fn create_impl_assoc_ty_alias(
    db: &RootDatabase,
    ty_alias: &hir::TypeAlias,
    where_pred_ty: &str,
) -> Option<ast::AssocItem> {
    let source = ty_alias.source(db)?.value;
    let where_clause = source.where_clause();
    // FIXME: Make sure all projections works with resolved Self.

    // Replacing all Self keywords should be fine.
    let assignment = source
        .ty()
        .map(|ty| (make::ty(&ty.to_string().replace("Self", where_pred_ty)), where_clause));

    let type_alias_definition = make::ty_alias(
        ty_alias.name(db).as_str()?,
        source.generic_param_list(),
        None,
        None,
        assignment,
    );

    Some(ast::AssocItem::TypeAlias(type_alias_definition))
}

fn create_impl_assoc_method(
    db: &RootDatabase,
    method: &hir::Function,
    variant_arms: &[(ast::Pat, ast::Path, hir::Variant)],
    _where_pred_ty: &str,
) -> Option<ast::AssocItem> {
    // FIXME: Allow for async/unsafe-ness?
    if method.is_async(db) || method.is_unsafe_to_call(db) {
        return None;
    }

    let method_source = method.source(db)?.value;
    let method_name = method_source.name()?.to_string();

    let method_generics =
        method_source.generic_param_list().as_ref().map(ToString::to_string).unwrap_or_default();

    let method_params_normalized = method_source.param_list()?.syntax().to_string();

    let method_ret_type = method_source.ret_type();

    let (qualifier_or_accessor, method_receiver_access) = if let Some(sp) = method.self_param(db) {
        (".", Some(sp.access(db)))
    } else {
        ("::", None)
    };

    // FIXME: We should resolve type aliases if for some reason there a Self::Output = Self binding.
    let method_callable_args = convert_param_list_to_arg_list(method_source.param_list()?);

    // FIXME: Crying emoji

    let method_ret_type_is_self = method_ret_type
        .as_ref()
        .map(ast::RetType::syntax)
        .map(SyntaxNode::children)
        .and_then(|mut children| children.find_map(ast::PathType::cast))
        .as_ref()
        .and_then(ast::PathType::path)
        .as_ref()
        .and_then(ast::Path::as_single_name_ref);

    let ret_is_owned_self = matches!(
        method_ret_type_is_self,
        Some(ret_name_ref) if ret_name_ref.Self_token().is_some()
    );

    let default_arm = if ret_is_owned_self {
        "variant => variant$0"
    } else if method_ret_type.is_none() {
        "_ => ()$0"
    } else {
        "_ => $0"
    };

    let method_return = method_ret_type.as_ref().map(ast::RetType::to_string).unwrap_or_default();

    let method_where_clause =
        method_source.where_clause().as_ref().map(ast::WhereClause::to_string).unwrap_or_default();

    // If method doesn't have a receiver, return early.
    if method_receiver_access.is_none() {
        let method_body = format!("{{\n        {_where_pred_ty}{qualifier_or_accessor}{method_name}{method_callable_args}\n    }}");
        let method_definition = format!(
            "fn {method_name}{method_generics}{method_params_normalized} {method_return} {method_where_clause}{method_body}"
        );

        return make::maybe_fn_(&method_definition).map(ast::AssocItem::Fn);
    }

    let match_arm_pats = variant_arms
        .iter()
        .enumerate()
        .map(|(i, (pat, name, variant))| {
            let indentation = if i == 0 { "" } else { "      " };
            match (method_receiver_access, ret_is_owned_self) {
                (Some(hir::Access::Owned), true)  => {
                    let fields = format_variant_fields(db, variant);

                    match pat {
                        ast::Pat::RecordPat(record) => {
                            format_record_pat(indentation, record, &fields, name, qualifier_or_accessor, &method_name, &method_callable_args)
                        }
                        ast::Pat::TupleStructPat(tuple) => {
                            format_tuple_struct_pat(indentation, tuple, &fields, name, qualifier_or_accessor, &method_name, &method_callable_args)
                        }
                        _ => unreachable!(
                            "Should not be possible to get here because of `create_partial_match_arm`"
                        ),
                    }
                }
                _ => format!("{indentation}{pat} => {name}{qualifier_or_accessor}{method_name}{method_callable_args}")
            }
        })
        .join(",\n      ");

    let method_body = format!(
            "{{\n        match self {{\n            {match_arm_pats},\n            {default_arm}\n        }}\n    }}",
        );

    let method_definition = format!(
        "fn {method_name}{method_generics}{method_params_normalized} {method_return} {method_where_clause}{method_body}"
    );

    make::maybe_fn_(&method_definition).map(ast::AssocItem::Fn)
}

fn create_partial_match_arm(
    sema: &'_ hir::Semantics<'_, RootDatabase>,
    where_pred_ty_str: &str,
    variant_hir: &hir::Variant,
    trait_hir: &hir::Trait,
    generic_args: &[hir::Type],
) -> Option<(ast::Pat, ast::Path, hir::Variant)> {
    let variant_name_hir = variant_hir.name(sema.db);
    let variant_name = variant_name_hir.as_str()?;

    let variant_path = make::path_from_text(&format!("Self::{variant_name}"));

    for field in variant_hir.fields(sema.db) {
        let field_ty = field.source(sema.db).and_then(get_type_from_field_source)?;

        if where_pred_ty_str != "Self" && field_ty.to_string() != where_pred_ty_str {
            continue;
        }

        let resolved_field_ty = resolve_path_type(
            sema,
            &field_ty.syntax().descendants().find_map(ast::PathType::cast)?,
        )?;

        // FIXME: This comparison is not reliable, i.e. it does not take into account canonical
        // matches.
        //
        // If we have two obligations: `A<i32>: Trait<i32, T>` and `A<i32>: Trait<i32, U>`.
        // and we have a `impl<E> Trait<i32, E> for A<i32>`, we would have to canonicalize the
        // obligations to be able to find out if that implmentation exists.
        //
        // That would look something like this: `A<i32>: Trait<i32, _>`.
        if field_ty.to_string() == where_pred_ty_str
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

fn get_sibling_token_text_size(node: &SyntaxNode, direction: Direction) -> TextSize {
    match direction {
        Direction::Prev => node
            .prev_sibling_or_token()
            .and_then(|sib| {
                (sib.kind() == SyntaxKind::WHITESPACE).then(|| sib.text_range().start())
            })
            .unwrap_or_else(|| node.text_range().start()),
        Direction::Next => node
            .next_sibling_or_token()
            .and_then(|sib| (sib.kind() == SyntaxKind::WHITESPACE).then(|| sib.text_range().end()))
            .unwrap_or_else(|| node.text_range().end()),
    }
}

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

fn format_record_pat(
    indentation: &str,
    record: &ast::RecordPat,
    fields: &str,
    name: &ast::Path,
    qualifier_or_accessor: &str,
    method_name: &str,
    method_callable_args: &ast::ArgList,
) -> String {
    let path = record.path().unwrap();
    let params = fields.replace(
        &name.to_string(),
        &format!("{name}: {name}{qualifier_or_accessor}{method_name}{method_callable_args}"),
    );
    format!("{indentation}{path} {{ {fields} }} => {path} {{ {params} }}")
}

fn format_tuple_struct_pat(
    indentation: &str,
    tuple: &ast::TupleStructPat,
    fields: &str,
    name: &ast::Path,
    qualifier_or_accessor: &str,
    method_name: &str,
    method_callable_args: &ast::ArgList,
) -> String {
    let path = tuple.path().unwrap();
    let params = fields.replace(
        &name.to_string(),
        &format!("{name}{qualifier_or_accessor}{method_name}{method_callable_args}"),
    );
    format!("{indentation}{path} ({fields}) => {path}({params})")
}

fn format_variant_fields(db: &RootDatabase, variant: &hir::Variant) -> String {
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

fn _equal_in_kinds_and_tokens(left: &SyntaxNode, right: &SyntaxNode) -> bool {
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

fn resolve_path_type(
    sema: &'_ hir::Semantics<'_, RootDatabase>,
    path: &ast::PathType,
) -> Option<hir::Type> {
    match sema.resolve_path(&path.path()?)? {
        hir::PathResolution::Def(def) => match def {
            hir::ModuleDef::BuiltinType(ty) => Some(ty.ty(sema.db)),
            hir::ModuleDef::Adt(ty) => Some(ty.ty(sema.db)),
            // FIXME: Need to verify this
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
        match self {
            Self::One { verycool, .. } => verycool.next(input),
            Self::Two(_, _, f2, ..) => f2.next(input),
            _ => $0
        }
    }

    fn next2()  {
        u32::next2()
    }
}"#,
        );
    }

    // ----------------------------------------------------------------------------
    #[test]
    fn test_trait_impl_with_self() {
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

enum Enum {
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
        match self {
            Self::V1 (f0, f1, f2) => Self::V1(f0, f1.method(input), f2),
            Self::V2 { name, age } => Self::V2 { name: name.method(input), age },
            Self::V3 { age, name } => Self::V3 { age: age.method(input), name },
            Self::v4 (f0, f1) => Self::v4(f0, f1.method(input)),
            variant => variant$0
        }
    }
}
"#,
        );
    }
    // ----------------------------------------------------------------------------
    #[test]
    fn test_macro_trait_impl() {
        check_assist(
            generate_trait_impl_for_enum,
            r#"
macro_rules! impl_Trait {
    ($NAME:ident $TY:ty) => {
        trait $NAME<T> {
            fn method(self) -> Self;
        }

        impl $NAME<$TY> for $TY {
            fn method(self) -> Self {
                self
            }
        }
    };
}

impl_Trait!(Trait u32);

enum Enum
where
    u32: Tr$0ait<u32>,
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
macro_rules! impl_Trait {
    ($NAME:ident $TY:ty) => {
        trait $NAME<T> {
            fn method(self) -> Self;
        }

        impl $NAME<$TY> for $TY {
            fn method(self) -> Self {
                self
            }
        }
    };
}

impl_Trait!(Trait u32);

enum Enum {
    V1(i32, u32, i32),
    V2 { name: u32, age: i32 },
    V3 { age: u32, name: u32 },
    v4(String, u32),
    v5(),
    v6 {},
    v7,
}

impl Trait<u32> for Enum
where
    u32: Trait<u32>,
{
    fn method(self) ->Self {
        match self {
            Self::V1 (f0, f1, f2) => Self::V1(f0, f1.method(), f2),
            Self::V2 { name, age } => Self::V2 { name: name.method(), age },
            Self::V3 { age, name } => Self::V3 { age: age.method(), name },
            Self::v4 (f0, f1) => Self::v4(f0, f1.method()),
            variant => variant$0
        }
    }
}
"#,
        );
    }
}
