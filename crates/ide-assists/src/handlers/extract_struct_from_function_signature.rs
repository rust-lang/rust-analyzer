use hir::{Function, ModuleDef};
use ide_db::{RootDatabase, assists::AssistId, path_transform::PathTransform};
use itertools::Itertools;
use syntax::{
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode, T,
    ast::{
        self, HasAttrs, HasGenericParams, HasName, HasVisibility,
        edit::{AstNodeEdit, IndentLevel},
        make,
    },
    match_ast,
    ted::{self, Element},
};

use crate::{AssistContext, Assists};
// Assist: extract_struct_from_function_signature
//
// Extracts a struct (part) of the signature of a function.
//
// ```
// fn foo($0bar: u32, baz: u32$0) { ... }
// ```
// ->
// ```
// struct FooStruct{ bar: u32, baz: u32 }
//
// fn foo(foo_struct: FooStruct) { ... }
// ```

pub(crate) fn extract_struct_from_function_signature(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let fn_ast = ctx.find_node_at_offset::<ast::Fn>()?;
    let param_list = fn_ast.param_list()?;
    let used_param_list = param_list
        .params()
        // filter to only parameters in selection
        .filter(|p| p.syntax().text_range().intersect(ctx.selection_trimmed()).is_some())
        .collect_vec();
    // TODO: make sure at least one thing there
    let target =
        used_param_list.iter().map(|p| p.syntax().text_range()).reduce(|t, t2| t.cover(t2))?;
    let fn_name = fn_ast.name()?;
    let name = make::name(&format!("{}Struct", stdx::to_camel_case(fn_name.text_non_mutable())));

    let fn_hir = ctx.sema.to_def(&fn_ast)?;
    if existing_definition(ctx.db(), &name, &fn_hir) {
        cov_mark::hit!(test_extract_function_signature_not_applicable_if_struct_exists);
        return None;
    }

    // TODO: special handiling for self?
    // TODO: special handling for destrutered types (or maybe just don't support code action on
    // destructed types yet

    let field_list = make::record_field_list(
        used_param_list
            .iter()
            .map(|param| {
                Some(make::record_field(
                    fn_ast.visibility(),
                    // only works if its an ident pattern
                    param.pat().and_then(pat_to_name)?,
                    // TODO: how are we going to handle references without explicit lifetimes
                    param.ty()?,
                ))
            })
            .collect::<Option<Vec<_>>>()?,
    );
    acc.add(
        AssistId::refactor_rewrite("extract_struct_from_function_signature"),
        "Extract struct from signature of a function",
        target,
        |builder| {
            // TODO: update calls to the function
            tracing::info!("extract_struct_from_function_signature: starting edit");
            builder.edit_file(ctx.vfs_file_id());
            // this has to be after the edit_file (order matters)
            // fn_ast and param_list must be "mut" for the effect to work on used_param_lsit
            let fn_ast = builder.make_mut(fn_ast);
            let param_list = builder.make_mut(param_list);
            let used_param_list = used_param_list.into_iter().map(|p| builder.make_mut(p)).collect_vec();
            tracing::info!("extract_struct_from_function_signature: editing main file");

            let generic_params = fn_ast
                .generic_param_list()
                .and_then(|known_generics| extract_generic_params(&known_generics, &field_list));
            tracing::info!("extract_struct_from_function_signature: collecting generics");
            let generics = generic_params.as_ref().map(|generics| generics.clone_for_update());

            // resolve GenericArg in field_list to actual type
            // we would get a query error from salsa, if we would use the field_list
            // I think it is because the field list is
            // constructed in new generation.
            // So I do the resolving while its still param list
            // and then apply it into record list after
            let field_list = if let Some((target_scope, source_scope)) =
                ctx.sema.scope(fn_ast.syntax()).zip(ctx.sema.scope(param_list.syntax()))
            {
                let field_list = field_list.reset_indent();
                let field_list =
                    PathTransform::generic_transformation(&target_scope, &source_scope)
                        .apply(field_list.syntax());
                match_ast! {
                    match field_list {
                        ast::RecordFieldList(field_list) => field_list,
                        _ => unreachable!(),
                    }
                }
            } else {
                field_list.clone_for_update()
            };
            tracing::info!("extract_struct_from_function_signature: collecting fields");
            let def = create_struct_def(name.clone(), &fn_ast, &used_param_list, &field_list, generics);
            tracing::info!("extract_struct_from_function_signature: creating struct");

            let indent = fn_ast.indent_level();
            let def = def.indent(indent);


            ted::insert_all(
                ted::Position::before(fn_ast.syntax()),
                vec![
                    def.syntax().clone().into(),
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                ],
            );
            tracing::info!("extract_struct_from_function_signature: inserting struct {def}");
            update_function(name,  generic_params.map(|g| g.clone_for_update()), &used_param_list).unwrap();
            tracing::info!("extract_struct_from_function_signature: updating function signature and parameter uses");
        },
    )
}

fn update_function(
    name: ast::Name,
    generics: Option<ast::GenericParamList>,
    used_param_list: &[ast::Param],
) -> Option<()> {
    let generic_args = generics
        .filter(|generics| generics.generic_params().count() > 0)
        .map(|generics| generics.to_generic_args());
    // FIXME: replace with a `ast::make` constructor
    let ty = match generic_args {
        Some(generic_args) => make::ty(&format!("{name}{generic_args}")),
        None => make::ty(&name.text()),
    };

    let param = make::param(
        // do we want to destructure the struct
        // makes it easier in that we would not have to update all the uses of the variables in
        // the function
        ast::Pat::RecordPat(make::record_pat(
            make::path_from_text(name.text_non_mutable()),
            used_param_list
                .iter()
                .map(|p| p.pat())
                .chain(std::iter::once(Some(ast::Pat::RestPat(make::rest_pat()))))
                .collect::<Option<Vec<_>>>()?,
        )),
        ty,
    )
    .clone_for_update();
    // TODO: will eventually need to handle self too

    let range = used_param_list.first()?.syntax().syntax_element()
        ..=used_param_list.last()?.syntax().syntax_element();
    ted::replace_all(range, vec![param.syntax().syntax_element()]);
    // no need update uses of parameters in function, because we destructure the struct
    Some(())
}

fn pat_to_name(pat: ast::Pat) -> Option<ast::Name> {
    match pat {
        ast::Pat::IdentPat(ident_pat) => ident_pat.name(),
        _ => None,
    }
}
fn create_struct_def(
    name: ast::Name,
    fn_ast: &ast::Fn,
    param_ast: &[ast::Param],
    field_list: &ast::RecordFieldList,
    generics: Option<ast::GenericParamList>,
) -> ast::Struct {
    let fn_vis = fn_ast.visibility();

    let insert_vis = |node: &'_ SyntaxNode, vis: &'_ SyntaxNode| {
        let vis = vis.clone_for_update();
        ted::insert(ted::Position::before(node), vis);
    };

    // for fields without any existing visibility, use visibility of enum
    let field_list: ast::FieldList = {
        if let Some(vis) = &fn_vis {
            field_list
                .fields()
                .filter(|field| field.visibility().is_none())
                .filter_map(|field| field.name())
                .for_each(|it| insert_vis(it.syntax(), vis.syntax()));
        }

        field_list.clone().into()
    };
    let field_list = field_list.indent(IndentLevel::single());

    let strukt = make::struct_(fn_vis, name, generics, field_list).clone_for_update();

    // take comments from only inside signature
    ted::insert_all(
        ted::Position::first_child_of(strukt.syntax()),
        take_all_comments(param_ast.iter()),
    );

    // TODO: this may not be correct as we shouldn't put all the attributes at the top
    // copy attributes from each parameter
    ted::insert_all(
        ted::Position::first_child_of(strukt.syntax()),
        param_ast
            .iter()
            .flat_map(|p| p.attrs())
            .flat_map(|it| {
                vec![it.syntax().clone_for_update().into(), make::tokens::single_newline().into()]
            })
            .collect(),
    );

    strukt
}
// Note: this also detaches whitespace after comments,
// since `SyntaxNode::splice_children` (and by extension `ted::insert_all_raw`)
// detaches nodes. If we only took the comments, we'd leave behind the old whitespace.
fn take_all_comments<'a>(node: impl Iterator<Item = &'a ast::Param>) -> Vec<SyntaxElement> {
    let mut remove_next_ws = false;
    node.flat_map(|p| p.syntax().children_with_tokens())
        .filter_map(move |child| match child.kind() {
            SyntaxKind::COMMENT => {
                remove_next_ws = true;
                child.detach();
                Some(child)
            }
            SyntaxKind::WHITESPACE if remove_next_ws => {
                remove_next_ws = false;
                child.detach();
                Some(make::tokens::single_newline().into())
            }
            _ => {
                remove_next_ws = false;
                None
            }
        })
        .collect()
}
fn extract_generic_params(
    known_generics: &ast::GenericParamList,
    field_list: &ast::RecordFieldList,
) -> Option<ast::GenericParamList> {
    let mut generics = known_generics.generic_params().map(|param| (param, false)).collect_vec();

    let tagged_one = field_list
        .fields()
        .filter_map(|f| f.ty())
        .fold(false, |tagged, ty| tag_generics_in_function_signature(&ty, &mut generics) || tagged);

    let generics = generics.into_iter().filter_map(|(param, tag)| tag.then_some(param));
    tagged_one.then(|| make::generic_param_list(generics))
}
fn tag_generics_in_function_signature(
    ty: &ast::Type,
    generics: &mut [(ast::GenericParam, bool)],
) -> bool {
    let mut tagged_one = false;

    for token in ty.syntax().descendants_with_tokens().filter_map(SyntaxElement::into_token) {
        for (param, tag) in generics.iter_mut().filter(|(_, tag)| !tag) {
            match param {
                ast::GenericParam::LifetimeParam(lt)
                    if matches!(token.kind(), T![lifetime_ident]) =>
                {
                    if let Some(lt) = lt.lifetime() {
                        if lt.text().as_str() == token.text() {
                            *tag = true;
                            tagged_one = true;
                            break;
                        }
                    }
                }
                param if matches!(token.kind(), T![ident]) => {
                    if match param {
                        ast::GenericParam::ConstParam(konst) => konst
                            .name()
                            .map(|name| name.text().as_str() == token.text())
                            .unwrap_or_default(),
                        ast::GenericParam::TypeParam(ty) => ty
                            .name()
                            .map(|name| name.text().as_str() == token.text())
                            .unwrap_or_default(),
                        ast::GenericParam::LifetimeParam(lt) => lt
                            .lifetime()
                            .map(|lt| lt.text().as_str() == token.text())
                            .unwrap_or_default(),
                    } {
                        *tag = true;
                        tagged_one = true;
                        break;
                    }
                }
                _ => (),
            }
        }
    }

    tagged_one
}
fn existing_definition(db: &RootDatabase, variant_name: &ast::Name, variant: &Function) -> bool {
    variant
        .module(db)
        .scope(db, None)
        .into_iter()
        .filter(|(_, def)| match def {
            // only check type-namespace
            hir::ScopeDef::ModuleDef(def) => matches!(
                def,
                ModuleDef::Module(_)
                    | ModuleDef::Adt(_)
                    | ModuleDef::Variant(_)
                    | ModuleDef::Trait(_)
                    | ModuleDef::TypeAlias(_)
                    | ModuleDef::BuiltinType(_)
            ),
            _ => false,
        })
        .any(|(name, _)| name.as_str() == variant_name.text().trim_start_matches("r#"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{check_assist, check_assist_not_applicable};

    #[test]
    fn test_extract_function_signature_not_applicable_if_struct_exists() {
        cov_mark::check!(test_extract_function_signature_not_applicable_if_struct_exists);
        check_assist_not_applicable(
            extract_struct_from_function_signature,
            r#"
struct OneStruct;
fn one($0x: u8, y: u32) {}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_single_parameters() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo($0bar: i32$0, baz: i32) {}
"#,
            r#"
struct FooStruct{ bar: i32 }

fn foo(FooStruct { bar, .. }: FooStruct, baz: i32) {}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_all_parameters() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo($0bar: i32, baz: i32$0) {}
"#,
            r#"
struct FooStruct{ bar: i32, baz: i32 }

fn foo(FooStruct { bar, baz, .. }: FooStruct) {}
"#,
        );
    }
}
