use ide_db::FxHashSet;
use itertools::Itertools;
use syntax::ast::{
    self, AstNode, GenericArgList, GenericParamList, HasGenericParams, HasName, HasTypeBounds,
    ParamList, TypeBoundList, WhereClause,
};

use crate::{AssistContext, AssistId, AssistKind, Assists};

// Assist: add_lifetime_to_function
//
// Adds a new lifetime to a struct, enum or union.
//
// ```
// fn print(s: &$0'a str) {
//     println!("{s}");
// }
// ```
// ->
// ```
// fn print<'a>(s: &'a str) {
//     println!("{s}");
// }
// ```
pub(crate) fn add_lifetime_to_function(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let function = ctx.find_node_at_offset::<ast::Fn>()?;
    let target = function.syntax().text_range();

    let lifetimes_used_in_signature = function
        .param_list()
        .into_iter()
        .flat_map(lifetimes_from_param_list)
        .chain(
            function.ret_type().into_iter().filter_map(|ty| ty.ty()).flat_map(lifetimes_from_type),
        )
        .chain(function.where_clause().into_iter().flat_map(lifetimes_from_where_clause));
    let lifetimes_in_generic_list: FxHashSet<_> = function
        .generic_param_list()
        .iter()
        .flat_map(|gparams| gparams.lifetime_params())
        .filter_map(|lifetime_param| lifetime_param.lifetime())
        .map(|lifetime| lifetime.to_string())
        .collect();

    let mut lifetimes_to_add = lifetimes_used_in_signature
        .filter(|life| life != "'static")
        .filter(|life| !lifetimes_in_generic_list.contains(life))
        .peekable();
    lifetimes_to_add.peek()?;

    acc.add(
        AssistId("add_lifetime_to_function", AssistKind::QuickFix),
        "Add lifetime to function",
        target,
        |edit| {
            let mut lifetimes_to_add: Vec<_> = lifetimes_to_add.collect();
            lifetimes_to_add.sort();
            lifetimes_to_add.dedup();
            let lifetime_list = lifetimes_to_add.into_iter().join(", ");
            match function.generic_param_list() {
                Some(gen_param) => {
                    if let Some(lifetime_end) = gen_param.lifetime_params().last() {
                        edit.insert(
                            lifetime_end.syntax().text_range().end(),
                            format!(", {lifetime_list}"),
                        );
                    } else if let Some(generic_start) = gen_param.generic_params().next() {
                        edit.insert(
                            generic_start.syntax().text_range().start(),
                            format!("{lifetime_list}, "),
                        );
                    }
                }
                None => {
                    if let Some(name) = function.name() {
                        edit.insert(name.syntax().text_range().end(), format!("<{lifetime_list}>"));
                    }
                }
            }
        },
    )
}

fn lifetimes_from_type(ty: ast::Type) -> Vec<String> {
    match ty {
        ast::Type::ArrayType(arr_ty) => arr_ty.ty().map(lifetimes_from_type).unwrap_or_default(),
        ast::Type::DynTraitType(dynt_ty) => dynt_ty
            // dyn Trait<'a, &'b Struct<'c>>
            //          ^^^^^^^^^^^^^^^^^^^^
            .type_bound_list()
            .iter()
            // dyn Trait<'a, &'b Struct<'c>>
            //           ^^, ^^^^^^^^^^^^^^
            .flat_map(|list| list.bounds())
            .flat_map(|bound| {
                bound
                    // dyn Trait<'a, &'b Struct<'c>>
                    //               ^^^^^^^^^^^^^^
                    .ty()
                    .into_iter()
                    // dyn Trait<'a, &'b Struct<'c>>
                    //                ^^        ^^
                    .flat_map(lifetimes_from_type)
                    // dyn Trait<'a, &'b Struct<'c>>
                    //           ^^
                    .chain(bound.lifetime().map(|life| life.to_string()))
            })
            .collect(),
        ast::Type::FnPtrType(fn_ptr_ty) => fn_ptr_ty
            // fn(param1: &'a str, param2: &'b dyn Trait<'c>) -> &'d str
            //   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            .param_list()
            .into_iter()
            // fn(param1: &'a str, param2: &'b dyn Trait<'c>) -> &'d str
            //    ^^^^^^^^^^^^^^^, ^^^^^^^^^^^^^^^^^^^^^^^^^
            .flat_map(|param_list| param_list.params())
            // fn(param1: &'a str, param2: &'b dyn Trait<'c>) -> &'d str
            //            ^^^^^^^,         ^^^^^^^^^^^^^^^^^
            .filter_map(|param| param.ty())
            // fn(param1: &'a str, param2: &'b dyn Trait<'c>) -> &'d str
            //             ^^,              ^^           ^^
            .flat_map(lifetimes_from_type)
            .chain(
                fn_ptr_ty
                    // fn(param1: &'a str, param2: &'b dyn Trait<'c>) -> &'d str
                    //                                                   ^^^^^^^
                    .ret_type()
                    .and_then(|ty| ty.ty())
                    .into_iter()
                    // fn(param1: &'a str, param2: &'b dyn Trait<'c>) -> &'d str
                    //                                                    ^^
                    .flat_map(lifetimes_from_type),
            )
            .collect(),
        // Note: we collect the lifetimes from where bounds that have an `for` restriction on them.
        // We could also add any lifetimes we find inside ForTypes to the `generic_param_list` of
        // the `for` bound, but we instead add them to the function; it's less confusing.
        ast::Type::ForType(for_ty) => {
            let for_lifetimes: FxHashSet<_> = for_ty
                // for<'life> T: Trait<'life, 'a>
                //    ^^^^^^^
                .generic_param_list()
                .into_iter()
                // for<'life> T: Trait<'life, 'a>
                //     ^^^^^
                .flat_map(lifetimes_from_generic_params)
                .collect();
            for_ty
                // for<'life> T: Trait<'life, 'a>
                //               ^^^^^^^^^^^^^^^^
                .ty()
                .into_iter()
                // for<'life> T: Trait<'life, 'a>
                //                     ^^^^^, ^^
                .flat_map(lifetimes_from_type)
                // If a type was declared on the `for<'a>` bound, then we should not report it as
                // missing.
                // for<'life> T: Trait<'life, 'a>
                //                            ^^
                .filter(|life| !for_lifetimes.contains(life))
                .collect()
        }
        ast::Type::ImplTraitType(impl_trait_ty) => impl_trait_ty
            .type_bound_list()
            .into_iter()
            .flat_map(lifetimes_from_type_bounds)
            .collect(),
        // _ doesn't have any lifetimes
        ast::Type::InferType(_) => vec![],
        // macro calls also don't have lifetimes
        ast::Type::MacroType(_) => vec![],
        // ! is equally blessed with no lifetimes
        ast::Type::NeverType(_) => vec![],
        ast::Type::ParenType(paren_ty) => {
            paren_ty.ty().map(lifetimes_from_type).unwrap_or_default()
        }
        ast::Type::PathType(path_ty) => path_ty
            .path()
            .unwrap()
            .segments()
            .flat_map(|seg| {
                seg.generic_arg_list()
                    .into_iter()
                    .flat_map(lifetimes_from_generic_args)
                    .chain(seg.param_list().into_iter().flat_map(lifetimes_from_param_list))
                    .chain(
                        seg.ret_type()
                            .and_then(|ty| ty.ty())
                            .into_iter()
                            .flat_map(lifetimes_from_type),
                    )
            })
            .collect(),
        ast::Type::PtrType(ptr_ty) => ptr_ty.ty().map(lifetimes_from_type).unwrap_or_default(),
        ast::Type::RefType(ref_ty) => ref_ty
            .lifetime()
            .iter()
            .map(|life| life.to_string())
            .chain(ref_ty.ty().map(lifetimes_from_type).unwrap_or_default())
            .collect(),
        ast::Type::SliceType(slice_ty) => {
            slice_ty.ty().map(lifetimes_from_type).unwrap_or_default()
        }
        ast::Type::TupleType(tpl_ty) => tpl_ty.fields().flat_map(lifetimes_from_type).collect(),
    }
}

fn lifetimes_from_generic_params(generic_params: GenericParamList) -> impl Iterator<Item = String> {
    generic_params
        .lifetime_params()
        .flat_map(|lifetime_param| lifetime_param.lifetime())
        .map(|lifetime| lifetime.to_string())
}

fn lifetimes_from_type_bounds(type_bounds: TypeBoundList) -> impl Iterator<Item = String> {
    type_bounds.bounds().flat_map(|bound| {
        bound
            .ty()
            .into_iter()
            .flat_map(lifetimes_from_type)
            .chain(bound.lifetime().map(|life| life.to_string()))
    })
}

fn lifetimes_from_param_list(param_list: ParamList) -> impl Iterator<Item = String> {
    param_list
        .params()
        .filter_map(|param| param.ty())
        .flat_map(lifetimes_from_type)
        .map(|life| life.to_string())
}

fn lifetimes_from_generic_args(generic_args: GenericArgList) -> Vec<String> {
    generic_args
        .generic_args()
        .flat_map(|arg| match arg {
            ast::GenericArg::AssocTypeArg(_assoc_ty_arg) => vec![],
            ast::GenericArg::ConstArg(_) => vec![],
            ast::GenericArg::LifetimeArg(lifetime_arg) => vec![lifetime_arg.to_string()],
            ast::GenericArg::TypeArg(ty_arg) => {
                ty_arg.ty().map(lifetimes_from_type).unwrap_or_default()
            }
        })
        .collect()
}

fn lifetimes_from_where_clause(clause: WhereClause) -> impl Iterator<Item = String> {
    clause.predicates().flat_map(|pred| {
        let declared_params: FxHashSet<_> =
            pred.generic_param_list().into_iter().flat_map(lifetimes_from_generic_params).collect();
        pred.ty()
            // for<'b> T: std::fmt::Display + 'a + 'b
            //         ^
            .into_iter()
            // for<'b> T: std::fmt::Display + 'a + 'b
            //
            .flat_map(lifetimes_from_type)
            // for<'b> T: std::fmt::Display + 'a + 'b
            //                                ^^,  ^^
            .chain(pred.lifetime().map(|life| life.to_string()))
            .chain(pred.type_bound_list().into_iter().flat_map(lifetimes_from_type_bounds))
            .filter(move |life| !declared_params.contains(life))
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_add_lifetime_to_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f(s: &$0'a str) {}"#,
            r#"fn f<'a>(s: &'a str) {}"#,
        );
    }

    #[test]
    fn test_dont_add_lifetime_to_good_function() {
        check_assist_not_applicable(add_lifetime_to_function, r#"fn f<'a>(s: &$0'a str) {}"#);
    }

    #[test]
    fn test_add_lifetime_to_generic_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<T>(s: &$0'a str, t: T) {}"#,
            r#"fn f<'a, T>(s: &'a str, t: T) {}"#,
        );
    }

    #[test]
    fn test_add_lifetime_to_lifetime_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<'b>(s: &$0'a str, s2: &'b str) {}"#,
            r#"fn f<'b, 'a>(s: &'a str, s2: &'b str) {}"#,
        );
    }

    #[test]
    fn test_add_lifetime_to_lifetime_generic_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<'a, T>(s: &'a str, s2: &$0'b T) {}"#,
            r#"fn f<'a, 'b, T>(s: &'a str, s2: &'b T) {}"#,
        );
    }

    #[test]
    fn test_add_position_2_lifetime_to_lifetime_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<'a>(s: &'a str, s2: &$0'b str) {}"#,
            r#"fn f<'a, 'b>(s: &'a str, s2: &'b str) {}"#,
        );
    }

    #[test]
    fn test_add_lifetime_for_where_bound() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<'a>(s: &str) where 'a: '$0b { }"#,
            r#"fn f<'a, 'b>(s: &str) where 'a: 'b { }"#,
        );
    }

    #[test]
    fn test_add_2_lifetimes_to_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f(s: &'a str, s2: &$0'b str) {}"#,
            r#"fn f<'a, 'b>(s: &'a str, s2: &'b str) {}"#,
        );
    }

    #[test]
    fn test_add_lifetime_to_function_for_bound() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<T>(s: &str) where for<'b> &'b S<'a, T>: std::fmt::Display + '$0a + 'b {}"#,
            r#"fn f<'a, T>(s: &str) where for<'b> &'b S<'a, T>: std::fmt::Display + 'a + 'b {}"#,
        );
        check_assist(
            add_lifetime_to_function,
            r#"fn g<T: 'static>(s: &str) where T: 'static, S<'a, T>: for<'b> Trait<'a$0, &'b T> { }"#,
            r#"fn g<'a, T: 'static>(s: &str) where T: 'static, S<'a, T>: for<'b> Trait<'a, &'b T> { }"#,
        );
    }

    #[test]
    fn test_add_lifetime_to_complicated_function() {
        check_assist(
            add_lifetime_to_function,
            r#"fn f<T>(id: T) where T: Displa$0y + 'a {}"#,
            r#"fn f<'a, T>(id: T) where T: Display + 'a {}"#,
        );
        check_assist(
            add_lifetime_to_function,
            r#"fn f(things: &[Thing<'$0a>]) {}"#,
            r#"fn f<'a>(things: &[Thing<'a>]) {}"#,
        );
        check_assist(
            add_lifetime_to_function,
            r#"fn f<T, ID>(id: ID) where ID: for<'db> Lookup<Database<'db> = dyn DefDatabase + 'db, Data = T> + '$0a {}"#,
            r#"fn f<'a, T, ID>(id: ID) where ID: for<'db> Lookup<Database<'db> = dyn DefDatabase + 'db, Data = T> + 'a {}"#,
        );
    }
}
