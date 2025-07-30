use hir::{Function, ModuleDef};
use ide_db::{RootDatabase, assists::AssistId, path_transform::PathTransform};
use itertools::Itertools;
use stdx::to_camel_case;
use syntax::{
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode, T,
    ast::{
        self, HasAttrs, HasGenericParams, HasName, HasVisibility,
        edit::{AstNodeEdit, IndentLevel},
        make,
    },
    match_ast, ted,
};

use crate::{AssistContext, Assists};
// Assist: extract_struct_from_function_signature
//
// Extracts a struct (part) of the signature of a function.
//
// ```
// fn foo(bar: u32, baz: u32) { ... }
// ```
// ->
// ```
// struct FooStruct {
//      bar: u32,
//      baz: u32,
// }
//
// fn foo(FooStruct) { ... }
// ```

pub(crate) fn extract_struct_from_function_signature(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    // TODO: get more specific than param list
    // how to get function name and param list/part of param list the is selected seperatly
    // or maybe just auto generate random name not based on function name?
    let fn_ast = ctx.find_node_at_offset::<ast::Fn>()?;
    let fn_name = fn_ast.name()?;

    let fn_hir = ctx.sema.to_def(&fn_ast)?;
    if existing_definition(ctx.db(), &fn_name, &fn_hir) {
        cov_mark::hit!(test_extract_function_signature_not_applicable_if_struct_exists);
        return None;
    }

    // TODO: does this capture parenthesis
    let target = fn_ast.param_list()?.syntax().text_range();
    // TODO: special handiling for self?
    // TODO: special handling for destrutered types (or maybe just don't suppurt code action on
    // destructed types yet
    let field_list = make::record_field_list(
        fn_ast
            .param_list()?
            .params()
            .map(|param| {
                Some(make::record_field(
                    fn_ast.visibility(),
                    param.pat().and_then(pat_to_name)?,
                    param.ty()?,
                ))
            })
            .collect::<Option<Vec<_>>>()?,
    );
    let name = make::name(&format!("{}Struct", to_camel_case(fn_name.text_non_mutable())));
    acc.add(
        AssistId::refactor_rewrite("extract_struct_from_function_signature"),
        "Extract struct from signature of a function",
        target,
        |builder| {
            builder.edit_file(ctx.vfs_file_id());

            let generic_params = fn_ast
                .generic_param_list()
                .and_then(|known_generics| extract_generic_params(&known_generics, &field_list));
            let generics = generic_params.as_ref().map(|generics| generics.clone_for_update());

            // resolve GenericArg in field_list to actual type
            let field_list = if let Some((target_scope, source_scope)) =
                ctx.sema.scope(fn_ast.syntax()).zip(ctx.sema.scope(field_list.syntax()))
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

            let def = create_struct_def(name.clone(), &fn_ast, &field_list, generics);

            let indent = fn_ast.indent_level();
            let def = def.indent(indent);

            ted::insert_all(
                ted::Position::before(fn_ast.syntax()),
                vec![
                    def.syntax().clone().into(),
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                ],
            );
        },
    )
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

    // take comments from variant
    ted::insert_all(
        ted::Position::first_child_of(strukt.syntax()),
        take_all_comments(fn_ast.syntax()),
    );

    // copy attributes from enum
    ted::insert_all(
        ted::Position::first_child_of(strukt.syntax()),
        fn_ast
            .attrs()
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
fn take_all_comments(node: &SyntaxNode) -> Vec<SyntaxElement> {
    let mut remove_next_ws = false;
    node.children_with_tokens()
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
