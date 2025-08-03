use std::ops::Range;

use hir::{Function, HasCrate, Module, ModuleDef};
use ide_db::{
    FxHashSet, RootDatabase,
    assists::AssistId,
    defs::Definition,
    helpers::mod_path_to_ast,
    imports::insert_use::{ImportScope, InsertUseConfig, insert_use},
    path_transform::PathTransform,
    search::FileReference,
    source_change::SourceChangeBuilder,
};
use itertools::Itertools;
use syntax::{
    AstNode, Edition, SyntaxElement, SyntaxKind, SyntaxNode, T,
    ast::{
        self, CallExpr, HasArgList, HasAttrs, HasGenericArgs, HasGenericParams, HasName,
        HasVisibility, RecordExprField,
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
// fn foo(FooStruct { bar, baz, .. }: FooStruct) { ... }
// ```

pub(crate) fn extract_struct_from_function_signature(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let find_node_at_offset = ctx.find_node_at_offset::<ast::Fn>()?;
    let fn_ast = find_node_at_offset;
    let param_list = fn_ast.param_list()?;
    let used_param_list = param_list
        .params()
        // filter to only parameters in selection
        .filter(|p| p.syntax().text_range().intersect(ctx.selection_trimmed()).is_some())
        .collect_vec();
    let target =
        used_param_list.iter().map(|p| p.syntax().text_range()).reduce(|t, t2| t.cover(t2))?;
    let fn_name = fn_ast.name()?;
    let name = make::name(&format!("{}Struct", stdx::to_camel_case(fn_name.text_non_mutable())));

    let fn_hir = ctx.sema.to_def(&fn_ast)?;
    if existing_definition(ctx.db(), &name, &fn_hir) {
        cov_mark::hit!(test_extract_function_signature_not_applicable_if_struct_exists);
        return None;
    }

    // TODO: (future)special handling for self
    // since it puts struct above function it invalid needs to go outside the the impl block
    // if uses self parameter and that is selected:
    // do we still keep in it in the impl block/does it matter what type of impl block it is (if its
    // a trait then probably not)
    // what should the name for self parameter be in the struct
    // also you would need to grab out any generics from that impl block itself and any where
    // clauses
    // we also need special handling for method calls

    // TODO: (future)special handling for destrutered types (or maybe just don't support code action on
    // destructed types yet

    let field_list = extract_field_list(&fn_ast, &used_param_list)?;

    let start_index = used_param_list.first()?.syntax().index();
    let end_index = used_param_list.last()?.syntax().index();
    let used_params_range = start_index..end_index + 1;
    acc.add(
        AssistId::refactor_rewrite("extract_struct_from_function_signature"),
        "Extract struct from signature of a function",
        target,
        |builder| {
            let  new_lifetime_count = field_list.fields().filter_map(|f|f.ty()).map(|t|new_life_time_count(&t)).sum();
            let edition = fn_hir.krate(ctx.db()).edition(ctx.db());
            let enum_module_def = ModuleDef::from(fn_hir);

            let usages = Definition::Function(fn_hir).usages(&ctx.sema).all();
            let mut visited_modules_set = FxHashSet::default();
            let current_module = fn_hir.module(ctx.db());
            visited_modules_set.insert(current_module);
            // record file references of the file the def resides in, we only want to swap to the edited file in the builder once

            let mut def_file_references = None;

            for (file_id, references) in usages {
                if file_id == ctx.file_id() {
                    def_file_references = Some(references);
                    continue;
                }
                builder.edit_file(file_id.file_id(ctx.db()));
                let processed = process_references(
                    ctx,
                    builder,
                    &mut visited_modules_set,
                    &enum_module_def,
                    references,
                    name.clone()
                );
                processed.into_iter().for_each(|(path, node, import)| {
                    apply_references(ctx.config.insert_use, path, node, import, edition, used_params_range.clone(), &field_list,
                        name.clone(),
                        // new_lifetime_count
                    );
                });
            }

            tracing::info!("extract_struct_from_function_signature: starting edit");
            builder.edit_file(ctx.vfs_file_id());
            let fn_ast_mut = builder.make_mut(fn_ast.clone());
             builder.make_mut(param_list.clone());
            let used_param_list = used_param_list.into_iter().map(|p| builder.make_mut(p)).collect_vec();
            tracing::info!("extract_struct_from_function_signature: editing main file");
            // this has to be after the edit_file (order matters)
            // fn_ast and param_list must be "mut" for the effect to work on used_param_list
            if let Some(references) = def_file_references {
                let processed = process_references(
                    ctx,
                    builder,
                    &mut visited_modules_set,
                    &enum_module_def,
                    references,
                    name.clone()
                );
                processed.into_iter().for_each(|(path, node, import)| {
                    apply_references(ctx.config.insert_use, path, node, import, edition, used_params_range.clone(), &field_list,
                        name.clone(),
                        // new_lifetime_count
                    );
                });
            }


            let generic_params = fn_ast
                .generic_param_list()
                .and_then(|known_generics| extract_generic_params(&known_generics, &field_list));
            tracing::info!("extract_struct_from_function_signature: collecting generics");
            let mut generics = generic_params.as_ref().map(|generics| generics.clone_for_update());

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
            field_list.fields().filter_map(|f|f.ty()).try_for_each(|t|generate_new_lifetimes(&t, &mut generics));
            tracing::info!("extract_struct_from_function_signature: collecting fields");
            let def = create_struct_def(name.clone(), &fn_ast_mut, &used_param_list, &field_list, generics);
            tracing::info!("extract_struct_from_function_signature: creating struct");

            let indent = fn_ast_mut.indent_level();
            let def = def.indent(indent);


            ted::insert_all(
                ted::Position::before(fn_ast_mut.syntax()),
                vec![
                    def.syntax().clone().into(),
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                ],
            );
            tracing::info!("extract_struct_from_function_signature: inserting struct {def}");
            update_function(name,  generic_params.map(|g| g.clone_for_update()), &used_param_list, new_lifetime_count).unwrap();
            tracing::info!("extract_struct_from_function_signature: updating function signature and parameter uses");
        },
    )
}

fn extract_field_list(
    fn_ast: &ast::Fn,
    used_param_list: &[ast::Param],
) -> Option<ast::RecordFieldList> {
    let field_list = make::record_field_list(
        used_param_list
            .iter()
            .map(|param| {
                Some(make::record_field(
                    fn_ast.visibility(),
                    // only works if its an ident pattern
                    param.pat().and_then(pat_to_name)?,
                    param.ty().filter(|ty| !contains_impl_trait(ty))?,
                ))
            })
            .collect::<Option<Vec<_>>>()?,
    );
    Some(field_list)
}

fn update_function(
    name: ast::Name,
    generics: Option<ast::GenericParamList>,
    used_param_list: &[ast::Param],
    new_lifetime_count: usize,
) -> Option<()> {
    let generic_args = generics
        .filter(|generics| generics.generic_params().count() > 0)
        .or((new_lifetime_count > 0).then_some(make::generic_param_list(std::iter::empty())))
        .map(|generics| {
            let args = generics.to_generic_args().clone_for_update();
            (0..new_lifetime_count).for_each(|_| {
                args.add_generic_arg(
                    make::lifetime_arg(make::lifetime("'_")).clone_for_update().into(),
                )
            });
            args
        });
    // FIXME: replace with a `ast::make` constructor
    let ty = match generic_args {
        Some(generic_args) => make::ty(&format!("{name}{generic_args}")),
        None => make::ty(&name.text()),
    };

    let param = make::param(
        // we destructure the struct
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

    // it is fine to unwrap() to because there is at least one parameter (if there is no parameters
    // the code action will not show)
    let start_index = used_param_list.first().unwrap().syntax().index();
    let end_index = used_param_list.last().unwrap().syntax().index();
    let used_params_range = start_index..end_index + 1;
    let new = vec![param.syntax().syntax_element()];
    used_param_list.first().unwrap().syntax().parent()?.splice_children(used_params_range, new);
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
fn generate_unique_lifetime_param_name(
    existing_type_param_list: &Option<ast::GenericParamList>,
) -> Option<ast::Lifetime> {
    match existing_type_param_list {
        Some(type_params) => {
            let used_lifetime_params: FxHashSet<_> =
                type_params.lifetime_params().map(|p| p.syntax().text().to_string()).collect();
            ('a'..='z').map(|it| format!("'{it}")).find(|it| !used_lifetime_params.contains(it))
        }
        None => Some("'a".to_owned()),
    }
    .map(|it| make::lifetime(&it))
}
fn new_life_time_count(ty: &ast::Type) -> usize {
    ty.syntax()
        .descendants()
        .filter_map(ast::Lifetime::cast)
        .filter(|lifetime| lifetime.text() == "'_")
        .count()
}
fn contains_impl_trait(ty: &ast::Type) -> bool {
    ty.syntax().descendants().any(|ty| ty.kind() == ast::ImplTraitType::kind())
}
fn generate_new_lifetimes(
    ty: &ast::Type,
    existing_type_param_list: &mut Option<ast::GenericParamList>,
) -> Option<()> {
    for token in ty.syntax().descendants() {
        if let Some(lt) = ast::Lifetime::cast(token.clone())
            && lt.text() == "'_"
        {
            let new_lt = generate_unique_lifetime_param_name(existing_type_param_list)?;
            existing_type_param_list
                .get_or_insert(make::generic_param_list(std::iter::empty()).clone_for_update())
                .add_generic_param(make::lifetime_param(new_lt.clone()).clone_for_update().into());

            ted::replace(lt.syntax(), new_lt.clone_for_update().syntax());
        }
    }
    Some(())
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

fn process_references(
    ctx: &AssistContext<'_>,
    builder: &mut SourceChangeBuilder,
    visited_modules: &mut FxHashSet<Module>,
    function_module_def: &ModuleDef,
    refs: Vec<FileReference>,
    name: ast::Name,
) -> Vec<(ast::PathSegment, SyntaxNode, Option<(ImportScope, hir::ModPath)>)> {
    // we have to recollect here eagerly as we are about to edit the tree we need to calculate the changes
    // and corresponding nodes up front
    let name = make::name_ref(name.text_non_mutable());
    refs.into_iter()
        .flat_map(|reference| {
            let (segment, scope_node, module) = reference_to_node(&ctx.sema, reference)?;
            let scope_node = builder.make_syntax_mut(scope_node);
            if !visited_modules.contains(&module) {
                let mod_path = module.find_use_path(
                    ctx.sema.db,
                    *function_module_def,
                    ctx.config.insert_use.prefix_kind,
                    ctx.config.import_path_config(),
                );
                if let Some(mut mod_path) = mod_path {
                    mod_path.pop_segment();
                    mod_path.push_segment(hir::Name::new_root(name.text_non_mutable()).clone());
                    let scope = ImportScope::find_insert_use_container(&scope_node, &ctx.sema)?;
                    visited_modules.insert(module);
                    return Some((segment, scope_node, Some((scope, mod_path))));
                }
            }
            Some((segment, scope_node, None))
        })
        .collect()
}
fn reference_to_node(
    sema: &hir::Semantics<'_, RootDatabase>,
    reference: FileReference,
) -> Option<(ast::PathSegment, SyntaxNode, hir::Module)> {
    // filter out the reference in macro (seems to be probalamtic with lifetimes/generics arguments)
    let segment =
        reference.name.as_name_ref()?.syntax().parent().and_then(ast::PathSegment::cast)?;

    // let segment_range = segment.syntax().text_range();
    // if segment_range != reference.range {
    //     return None;
    // }

    let parent = segment.parent_path().syntax().parent()?;
    let expr_or_pat = match_ast! {
        match parent {
            ast::PathExpr(_it) => parent.parent()?,
            ast::RecordExpr(_it) => parent,
            ast::TupleStructPat(_it) => parent,
            ast::RecordPat(_it) => parent,
            _ => return None,
        }
    };
    let module = sema.scope(&expr_or_pat)?.module();

    Some((segment.clone_for_update(), expr_or_pat, module))
}

fn apply_references(
    insert_use_cfg: InsertUseConfig,
    segment: ast::PathSegment,
    node: SyntaxNode,
    import: Option<(ImportScope, hir::ModPath)>,
    edition: Edition,
    used_params_range: Range<usize>,
    field_list: &ast::RecordFieldList,
    name: ast::Name,
    // new_lifetime_count: usize,
) -> Option<()> {
    if let Some((scope, path)) = import {
        insert_use(&scope, mod_path_to_ast(&path, edition), &insert_use_cfg);
    }
    // TODO: figure out lifetimes in referecnecs
    // becauuse we have to convert from segment being non turbofish, also only need
    // generics/lifetimes that are used in struct possibly not all the no ones for the original call
    // if no specified lifetimes/generics we just give empty one
    // if new_lifetime_count > 0 {
    //     (0..new_lifetime_count).for_each(|_| {
    //         segment
    //             .get_or_create_generic_arg_list()
    //             .add_generic_arg(make::lifetime_arg(make::lifetime("'_")).clone_for_update().into())
    //     });
    // }

    // current idea: the lifetimes can be inferred from the call
    if let Some(generics) = segment.generic_arg_list() {
        ted::remove(generics.syntax());
    }
    ted::replace(segment.name_ref()?.syntax(), name.clone_for_update().syntax());
    // deep clone to prevent cycle
    let path = make::path_from_segments(std::iter::once(segment.clone_subtree()), false);
    // TODO: do I need to to method call to
    let call = CallExpr::cast(node)?;
    let fields = make::record_expr_field_list(
        call.arg_list()?
            .args()
            .skip(used_params_range.start - 1)
            .take(used_params_range.end - used_params_range.start)
            .zip(field_list.fields())
            .map(|e| {
                e.1.name().map(|name| -> RecordExprField {
                    make::record_expr_field(make::name_ref(name.text_non_mutable()), Some(e.0))
                })
            })
            .collect::<Option<Vec<_>>>()?,
    );
    let record_expr = make::record_expr(path, fields).clone_for_update();

    call.arg_list()?
        .syntax()
        .splice_children(used_params_range, vec![record_expr.syntax().syntax_element()]);
    Some(())
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
    fn test_extract_function_signature_single_parameter() {
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
    #[test]
    fn test_extract_function_signature_all_parameters_with_reference() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo($0bar: i32, baz: i32$0) {}

fn main() {
    foo(1, 2)
}
"#,
            r#"
struct FooStruct{ bar: i32, baz: i32 }

fn foo(FooStruct { bar, baz, .. }: FooStruct) {}

fn main() {
    foo(FooStruct { bar: 1, baz: 2 })
}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_single_parameter_with_reference_separate_and_in_self() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
mod a {
    pub fn foo($0bar: i32$0, baz: i32) {
        foo(1, 2)
    }
}

mod b {
    use crate::a::foo;

    fn main() {
        foo(1, 2)
    }
}
"#,
            r#"
mod a {
    pub struct FooStruct{ pub bar: i32 }

    pub fn foo(FooStruct { bar, .. }: FooStruct, baz: i32) {
        foo(FooStruct { bar: 1 }, 2)
    }
}

mod b {
    use crate::a::{foo, FooStruct};

    fn main() {
        foo(FooStruct { bar: 1 }, 2)
    }
}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_single_parameter_with_reference() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
    fn foo($0bar: i32$0, baz: i32) {}

    fn main() {
        foo(1, 2)
    }
    "#,
            r#"
    struct FooStruct{ bar: i32 }

    fn foo(FooStruct { bar, .. }: FooStruct, baz: i32) {}

    fn main() {
        foo(FooStruct { bar: 1 }, 2)
    }
    "#,
        );
    }

    #[test]
    fn test_extract_function_signature_single_parameter_generic() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo<'a, A>($0bar: &'a A$0, baz: i32) {}
"#,
            r#"
struct FooStruct<'a, A>{ bar: &'a A }

fn foo<'a, A>(FooStruct { bar, .. }: FooStruct<'a, A>, baz: i32) {}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_single_parameter_generic_with_reference_in_self() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo<'a, A>($0bar: &'a A$0, baz: i32) {
    foo(1, 2)
}
"#,
            r#"
struct FooStruct<'a, A>{ bar: &'a A }

fn foo<'a, A>(FooStruct { bar, .. }: FooStruct<'a, A>, baz: i32) {
    foo(FooStruct { bar: 1 }, 2)
}
"#,
        );
    }

    #[test]
    fn test_extract_function_signature_single_parameter_anonymous_lifetime() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo($0bar: &'_ i32$0, baz: i32) {}
"#,
            r#"
struct FooStruct<'a>{ bar: &'a i32 }

fn foo(FooStruct { bar, .. }: FooStruct<'_>, baz: i32) {}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_single_parameter_anonymous_and_normal_lifetime() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo<'a>($0bar: &'_ &'a i32$0, baz: i32) {}
"#,
            r#"
struct FooStruct<'a, 'b>{ bar: &'b &'a i32 }

fn foo<'a>(FooStruct { bar, .. }: FooStruct<'a, '_>, baz: i32) {}
"#,
        );
    }

    #[test]
    fn test_extract_function_signature_single_parameter_anonymous_and_normal_lifetime_with_reference_in_self()
     {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo<'a>($0bar: &'_ &'a i32$0, baz: i32) {
    foo(bar, baz)
}
"#,
            r#"
struct FooStruct<'a, 'b>{ bar: &'b &'a i32 }

fn foo<'a>(FooStruct { bar, .. }: FooStruct<'a, '_>, baz: i32) {
    foo(FooStruct { bar: bar }, baz)
}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_not_applicable_with_impl_trait() {
        check_assist_not_applicable(
            extract_struct_from_function_signature,
            r"fn foo($0i: impl ToString) {  }",
        );
    }
}
