use std::ops::Range;

use hir::{HasCrate, Module, ModuleDef};
use ide_db::{
    FileId,
    FxHashSet,
    RootDatabase,
    assists::AssistId,
    defs::Definition,
    helpers::mod_path_to_ast,
    // this relies on ted
    imports::insert_use::{ImportScope, InsertUseConfig, insert_use},
    path_transform::PathTransform,
    search::FileReference,
    source_change::SourceChangeBuilder,
};
use itertools::Itertools;
use syntax::{
    AstNode, Edition, NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    algo::find_node_at_range,
    ast::{
        self, HasArgList, HasAttrs, HasGenericParams, HasName, HasVisibility, RecordField, make,
        syntax_factory::SyntaxFactory,
    },
    match_ast,
    syntax_editor::{Element, Position, SyntaxEditor},
};
use syntax::{T, ast::edit::AstNodeEdit};

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
// struct FooStruct { bar: u32, baz: u32 }
//
// fn foo(FooStruct { bar, baz, .. }: FooStruct) { ... }
// ```

pub(crate) fn extract_struct_from_function_signature(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let find_node_at_offset = ctx.find_node_at_offset::<ast::Fn>()?;
    let func = find_node_at_offset;
    let param_list = func.param_list()?;
    let used_param_list = param_list
        .params()
        // filter to only parameters in selection
        .filter(|p| p.syntax().text_range().intersect(ctx.selection_trimmed()).is_some())
        .collect_vec();
    let target =
        used_param_list.iter().map(|p| p.syntax().text_range()).reduce(|t, t2| t.cover(t2))?;
    let fn_name = func.name()?;
    let name = make::name(&format!("{}Struct", stdx::to_camel_case(fn_name.text_non_mutable())));

    let fn_hir = ctx.sema.to_def(&func)?;
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

    // TODO: (future)special handling for destrutered types (right now we don't support code action on
    // destructed types yet

    let field_list = extract_field_list(&func, &used_param_list)?;

    let start_idx = used_param_list.first()?.syntax().index();
    let end_idx = used_param_list.last()?.syntax().index();
    let used_params_range = start_idx..end_idx + 1;
    acc.add(
        AssistId::refactor_rewrite("extract_struct_from_function_signature"),
        "Extract struct from signature of a function",
        target,
        |builder| {
            let make = SyntaxFactory::with_mappings();
            let mut editor = builder.make_editor(func.syntax());
            let n_new_lifetimes =
                field_list.iter().filter_map(|f| f.ty()).map(|t| new_life_time_count(&t)).sum();
            let edition = fn_hir.krate(ctx.db()).edition(ctx.db());
            let function_module_def = ModuleDef::from(fn_hir);

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
                let processed = process_references(
                    ctx,
                    &mut visited_modules_set,
                    &function_module_def,
                    references,
                    name.clone(),
                );
                processed.into_iter().for_each(|(path, import)| {
                    apply_references(
                        builder,
                        ctx.config.insert_use,
                        path,
                        import,
                        edition,
                        used_params_range.clone(),
                        field_list.iter(),
                        name.clone(),
                        file_id.file_id(ctx.db()),
                    );
                });
            }

            tracing::info!("extract_struct_from_function_signature: starting edit");
            // if in impl block then put struct before the impl block
            let (indent, syntax) = param_list
                .self_param()
                .and_then(|_| ctx.find_node_at_range::<ast::Impl>())
                .map(|imp| (imp.indent_level(), imp.syntax().clone()))
                .unwrap_or((func.indent_level(), func.syntax().clone()));
            tracing::info!("extract_struct_from_function_signature: editing main file");
            if let Some(references) = def_file_references {
                let processed = process_references(
                    ctx,
                    &mut visited_modules_set,
                    &function_module_def,
                    references,
                    name.clone(),
                );
                processed.into_iter().for_each(|(path, import)| {
                    apply_references(
                        builder,
                        ctx.config.insert_use,
                        path,
                        import,
                        edition,
                        used_params_range.clone(),
                        field_list.iter(),
                        name.clone(),
                        ctx.vfs_file_id(),
                    );
                });
            }

            let generic_params = func.generic_param_list().and_then(|known_generics| {
                extract_generic_params(&known_generics, field_list.iter())
            });
            tracing::info!("extract_struct_from_function_signature: collecting generics");
            let  generics = generic_params.clone();

            // resolve GenericArg in field_list to actual type
            // we would get a query error from salsa, if we would use the field_list
            // I think it is because the field list is
            // constructed in new generation.
            // So I do the resolving while its still param list
            // and then apply it into record list after
            let field_list = make.record_field_list(field_list).clone_subtree();
            let mut field_editor = SyntaxEditor::new(field_list.syntax().clone());
            let mut generic_editor = generics.map(|generics| {
                let syntax_editor = SyntaxEditor::new(generics.clone_subtree().syntax().clone());
                (generics, syntax_editor)
            });
            field_list
                .fields()
                .flat_map(|f| f.ty())
                .try_for_each(|ty| generate_new_lifetimes(&mut field_editor, &ty, &mut generic_editor));
            let generics = generic_editor.map(|(_, editor)| {let binding = editor.finish();
            let generics = binding.new_root();
                match_ast! {match generics {

                        ast::GenericParamList(generics) => generics,
                        _ => unreachable!(),

                }}
            } );


            move_comments_and_attributes(&mut field_editor, &used_param_list, &field_list);

            let field_list = if let Some((target_scope, source_scope)) =
                ctx.sema.scope(func.syntax()).zip(ctx.sema.scope(param_list.syntax()))
            {
                let field_list =
                    PathTransform::generic_transformation(&target_scope, &source_scope)
                        .apply(field_editor.finish().new_root());
                match_ast! {
                    match field_list {
                        ast::RecordFieldList(field_list) => field_list,
                        _ => unreachable!(),
                    }
                }
            } else {
                field_list
            };
            tracing::info!("extract_struct_from_function_signature: collecting fields");
            let def = create_struct_def(name.clone(), &func, &field_list, generics);
            tracing::info!("extract_struct_from_function_signature: creating struct");
            let def = def.indent(indent);
            editor.insert_all(
                Position::before(syntax),
                vec![
                    def.syntax().clone().into(),
                    make::tokens::whitespace(&format!("\n\n{indent}")).into(),
                ],
            );
            tracing::info!("extract_struct_from_function_signature: inserting struct {def}");
            update_function(&mut editor, name, generic_params, &used_param_list, n_new_lifetimes)
                .unwrap();
            tracing::info!(
                "extract_struct_from_function_signature: updating function signature and parameter uses"
            );
            editor.add_mappings(make.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), editor);
        },
    )
}

fn extract_field_list(func: &ast::Fn, used_param_list: &[ast::Param]) -> Option<Vec<RecordField>> {
    used_param_list
        .iter()
        .map(|param| {
            Some(make::record_field(
                func.visibility(),
                // only works if its an ident pattern
                param.pat().and_then(pat_to_name)?,
                param.ty().filter(|ty| !contains_impl_trait(ty))?,
            ))
        })
        .collect::<Option<Vec<_>>>()
}

fn update_function(
    editor: &mut SyntaxEditor,
    name: ast::Name,
    generics: Option<ast::GenericParamList>,
    used_param_list: &[ast::Param],
    n_new_lifetimes: usize,
) -> Option<()> {
    let generic_args = generics
        .filter(|generics| generics.generic_params().count() > 0)
        .or((n_new_lifetimes > 0).then_some(make::generic_param_list(std::iter::empty())))
        .map(move |generics| {
            let args = generics.to_generic_args().clone_subtree();
            let mut editor = SyntaxEditor::new(args.syntax().clone());
            (0..n_new_lifetimes).for_each(|_| {
                editor.add_generic_arg(
                    &args,
                    make::lifetime_arg(make::lifetime("'_")).clone_for_update().into(),
                );
            });
            editor.finish().new_root().clone()
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
    let start_idx = used_param_list.first().unwrap().syntax();
    let end_idx = used_param_list.last().unwrap().syntax();
    let new = vec![param.syntax().syntax_element()];
    editor.replace_all(
        NodeOrToken::Node(start_idx.clone())..=NodeOrToken::Node(end_idx.clone()),
        new,
    );
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
    func: &ast::Fn,
    field_list: &ast::RecordFieldList,
    generics: Option<ast::GenericParamList>,
) -> ast::Struct {
    let fn_vis = func.visibility();

    make::struct_(fn_vis, name, generics, field_list.clone().into())
}

fn move_comments_and_attributes(
    editor: &mut SyntaxEditor,
    param_ast: &[ast::Param],
    field_list: &ast::RecordFieldList,
) {
    // if we do not expleictly copy over comments/attribures they just get lost
    // TODO: what about comments/attributes in between parameters
    param_ast.iter().zip(field_list.fields()).for_each(|(param, field)| {
        let param = param.clone_for_update();
        let elements = take_all_comments(param.clone());
        editor.insert_all(Position::first_child_of(field.syntax()), elements);
        editor.insert_all(
            Position::first_child_of(field.syntax()),
            param
                .attrs()
                .flat_map(|it| [it.syntax().clone().into(), make::tokens::single_newline().into()])
                .collect(),
        );
    })
}
fn take_all_comments(node: impl ast::AstNode) -> Vec<SyntaxElement> {
    let mut remove_next_ws = false;
    node.syntax()
        .children_with_tokens()
        .filter_map(move |child| match child.kind() {
            SyntaxKind::COMMENT => {
                remove_next_ws = true;
                Some(child)
            }
            SyntaxKind::WHITESPACE if remove_next_ws => {
                remove_next_ws = false;
                Some(make::tokens::single_newline().into())
            }
            _ => {
                remove_next_ws = false;
                None
            }
        })
        .collect()
}
fn extract_generic_params<'a>(
    known_generics: &ast::GenericParamList,
    field_list: impl Iterator<Item = &'a RecordField>,
) -> Option<ast::GenericParamList> {
    let mut generics = known_generics.generic_params().map(|param| (param, false)).collect_vec();

    let tagged_one = field_list
        .filter_map(|f| f.ty())
        .fold(false, |tagged, ty| tag_generics_in_function_signature(&ty, &mut generics) || tagged);

    let generics = generics.into_iter().filter_map(|(param, tag)| tag.then_some(param));
    tagged_one.then(|| make::generic_param_list(generics))
}
fn generate_unique_lifetime_param_name(
    existing_type_param_list: &Option<(ast::GenericParamList, SyntaxEditor)>,
) -> Option<ast::Lifetime> {
    match existing_type_param_list {
        Some((type_params, _)) => {
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
        .filter(|t| {
            match_ast! { match t {
                ast::Lifetime(lt) => lt.text() == "'_",
                ast::RefType(r) => r.lifetime().is_none(),
                _ => false
            }}
        })
        .count()
}
fn contains_impl_trait(ty: &ast::Type) -> bool {
    ty.syntax().descendants().any(|ty| ty.kind() == ast::ImplTraitType::kind())
}
fn generate_new_lifetimes(
    fields_editor: &mut SyntaxEditor,
    ty: &ast::Type,
    existing_type_param_list: &mut Option<(ast::GenericParamList, SyntaxEditor)>,
) -> Option<()> {
    for token in ty.syntax().descendants() {
        // we do not have to worry about for<'a> because we are only looking at '_ or &Type
        // if you have an unbound lifetime thats on you
        if let Some(lt) = ast::Lifetime::cast(token.clone())
            && lt.text() == "'_"
        {
            let new_lt = generate_unique_lifetime_param_name(existing_type_param_list)?;
            fields_editor.replace(lt.syntax(), new_lt.syntax().clone_for_update());
            let (generics, editor) = existing_type_param_list.get_or_insert_with(|| {
                let generics = make::generic_param_list(std::iter::empty());
                let syntax_editor = SyntaxEditor::new(generics.syntax().clone_subtree());

                (generics, syntax_editor)
            });
            editor.add_generic_param(
                generics,
                make::lifetime_param(new_lt).clone_for_update().into(),
            );
        } else if let Some(r) = ast::RefType::cast(token.clone())
            && r.lifetime().is_none()
        {
            let new_lt = generate_unique_lifetime_param_name(existing_type_param_list)?;
            fields_editor.insert_all(
                Position::after(r.amp_token()?),
                vec![
                    new_lt.syntax().clone_for_update().into(),
                    make::tokens::whitespace(" ").into(),
                ],
            );
            let (generics, editor) = existing_type_param_list.get_or_insert_with(|| {
                let generics = make::generic_param_list(std::iter::empty());
                let syntax_editor = SyntaxEditor::new(generics.syntax().clone_subtree());

                (generics, syntax_editor)
            });

            editor.add_generic_param(
                generics,
                make::lifetime_param(new_lt).clone_for_update().into(),
            );
        }
        // TODO: nominal types that have only lifetimes
        // struct Bar<'a, 'b> { f: &'a &'b i32 }
        // fn foo(f: Bar) {}
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
                    if let Some(lt) = lt.lifetime()
                        && lt.text().as_str() == token.text()
                    {
                        *tag = true;
                        tagged_one = true;
                        break;
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
fn existing_definition(
    db: &RootDatabase,
    variant_name: &ast::Name,
    variant: &hir::Function,
) -> bool {
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
    visited_modules: &mut FxHashSet<Module>,
    function_module_def: &ModuleDef,
    refs: Vec<FileReference>,
    name: ast::Name,
) -> Vec<(CallExpr, Option<(ImportScope, hir::ModPath)>)> {
    // we have to recollect here eagerly as we are about to edit the tree we need to calculate the changes
    // and corresponding nodes up front
    let name = make::name_ref(name.text_non_mutable());
    refs.into_iter()
        .flat_map(|reference| {
            let (call, scope_node, module) = reference_to_node(&ctx.sema, reference)?;
            if !visited_modules.contains(&module) {
                let mod_path = module.find_use_path(
                    ctx.sema.db,
                    *function_module_def,
                    ctx.config.insert_use.prefix_kind,
                    ctx.config.find_path_confg(ctx.sema.is_nightly(module.krate())),
                );
                if let Some(mut mod_path) = mod_path {
                    mod_path.pop_segment();
                    mod_path.push_segment(hir::Name::new_root(name.text_non_mutable()).clone());
                    let scope = ImportScope::find_insert_use_container(&scope_node, &ctx.sema)?;
                    visited_modules.insert(module);
                    return Some((call, Some((scope, mod_path))));
                }
            }
            Some((call, None))
        })
        .collect()
}
fn reference_to_node(
    sema: &hir::Semantics<'_, RootDatabase>,
    reference: FileReference,
) -> Option<(CallExpr, SyntaxNode, hir::Module)> {
    // find neareat method call/call to the reference because different amount of parents between
    // name and full call depending on if its method call or normal call
    let node =
        find_node_at_range::<CallExpr>(reference.name.as_name_ref()?.syntax(), reference.range)?;

    // let segment_range = segment.syntax().text_range();
    // if segment_range != reference.range {
    //     return None;
    // }

    let module = sema.scope(node.syntax())?.module();

    Some((node.clone(), node.syntax().clone(), module))
}

fn apply_references<'a>(
    builder: &mut SourceChangeBuilder,
    insert_use_cfg: InsertUseConfig,
    call: CallExpr,
    import: Option<(ImportScope, hir::ModPath)>,
    edition: Edition,
    used_params_range: Range<usize>,
    field_list: impl Itertools<Item = &'a RecordField>,
    name: ast::Name,
    file_id: impl Into<FileId>,
) -> Option<()> {
    let mut editor = builder.make_editor(call.syntax());
    if let Some((scope, path)) = import {
        let scope = builder.make_import_scope_mut(scope);
        // uses ted
        insert_use(&scope, mod_path_to_ast(&path, edition), &insert_use_cfg);
    }

    // current idea: the lifetimes can be inferred from the call
    let path = make::path_from_text(name.text_non_mutable());
    let fields = make::record_expr_field_list(
        call.arg_list()?
            .args()
            .skip(match call {
                // for some reason the indices for parameters of method go in increments of 3s (but
                // start at 4 to accommodate the self parameter)
                CallExpr::Method(_) => used_params_range.start / 3 - 1,
                CallExpr::Normal(_) => used_params_range.start - 1,
            })
            // the zip implicitly makes that it will only take the amount of parameters required
            .zip(field_list)
            .map(|e| {
                e.1.name().map(|name| -> ast::RecordExprField {
                    make::record_expr_field(make::name_ref(name.text_non_mutable()), Some(e.0))
                })
            })
            .collect::<Option<Vec<_>>>()?,
    );
    let first = call.arg_list()?.args().nth(match call {
        // for some reason the indices for parameters of method go in increments of 3s (but
        // start at 4 to accommodate the self parameter)
        CallExpr::Method(_) => used_params_range.start / 3 - 1,
        CallExpr::Normal(_) => used_params_range.start - 1,
    })?;
    let last = call.arg_list()?.args().nth(match call {
        // for some reason the indices for parameters of method go in increments of 3s (but
        // start at 4 to accommodate the self parameter)
        CallExpr::Method(_) => used_params_range.end / 3 - 1,
        CallExpr::Normal(_) => used_params_range.end / 3,
    })?;
    let record_expr = make::record_expr(path, fields).clone_for_update();
    editor.replace_all(
        NodeOrToken::Node(first.syntax().clone())..=NodeOrToken::Node(last.syntax().clone()),
        vec![record_expr.syntax().syntax_element()],
    );
    builder.add_file_edits(file_id, editor);
    Some(())
}

#[derive(Debug, Clone)]
enum CallExpr {
    Normal(ast::CallExpr),
    Method(ast::MethodCallExpr),
}
impl AstNode for CallExpr {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == ast::MethodCallExpr::kind() && kind == ast::CallExpr::kind()
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        ast::CallExpr::cast(syntax.clone())
            .map(CallExpr::Normal)
            .or(ast::MethodCallExpr::cast(syntax).map(CallExpr::Method))
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            CallExpr::Normal(call_expr) => call_expr.syntax(),
            CallExpr::Method(method_call_expr) => method_call_expr.syntax(),
        }
    }
}
impl HasArgList for CallExpr {
    fn arg_list(&self) -> Option<ast::ArgList> {
        match self {
            CallExpr::Normal(call_expr) => call_expr.arg_list(),
            CallExpr::Method(method_call_expr) => method_call_expr.arg_list(),
        }
    }
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
struct FooStruct { bar: i32 }

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
struct FooStruct { bar: i32, baz: i32 }

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
struct FooStruct { bar: i32, baz: i32 }

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
    pub struct FooStruct { pub bar: i32 }

    pub fn foo(FooStruct { bar, .. }: FooStruct, baz: i32) {
        foo(FooStruct { bar: 1 }, 2)
    }
}

mod b {
    use crate::a::{FooStruct, foo};

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
    struct FooStruct { bar: i32 }

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
struct FooStruct<'a, A> { bar: &'a A }

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
struct FooStruct<'a, A> { bar: &'a A }

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
struct FooStruct<'a> { bar: &'a i32 }

fn foo(FooStruct { bar, .. }: FooStruct<'_>, baz: i32) {}
"#,
        );
    }

    #[test]
    fn test_extract_function_signature_single_parameter_with_plain_reference_type() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo($0bar: &i32$0, baz: i32) {}
"#,
            r#"
struct FooStruct<'a> { bar: &'a i32 }

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
struct FooStruct<'a, 'b> { bar: &'b &'a i32 }

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
struct FooStruct<'a, 'b> { bar: &'b &'a i32 }

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
    #[test]
    fn test_extract_function_signature_in_method() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
struct Foo
impl Foo {
    fn foo(&self, $0j: i32, i: i32$0, z:i32) {  }
}

fn bar() {
    Foo.foo(1, 2, 3)
}
"#,
            r#"
struct Foo
struct FooStruct { j: i32, i: i32 }

impl Foo {
    fn foo(&self, FooStruct { j, i, .. }: FooStruct, z:i32) {  }
}

fn bar() {
    Foo.foo(FooStruct { j: 1, i: 2 }, 3)
}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_in_method_with_reference_in_impl() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
struct Foo
impl Foo {
    fn foo(&self, $0j: i32, i: i32$0, z:i32) {  }
    fn baz(&self) {
        self.foo(4, 5, 6)
    }
}

fn bar() {
    Foo.foo(1, 2, 3)
}
"#,
            r#"
struct Foo
struct FooStruct { j: i32, i: i32 }

impl Foo {
    fn foo(&self, FooStruct { j, i, .. }: FooStruct, z:i32) {  }
    fn baz(&self) {
        self.foo(FooStruct { j: 4, i: 5 }, 6)
    }
}

fn bar() {
    Foo.foo(FooStruct { j: 1, i: 2 }, 3)
}
"#,
        );
    }
    #[test]
    fn test_extract_function_signature_in_method_comments_and_attributes() {
        check_assist(
            extract_struct_from_function_signature,
            r#"
fn foo(
    #[foo]
    // gag
    $0f: i32,
) { }
"#,
            r#"
struct FooStruct { #[foo]
// gag
f: i32 }

fn foo(
    FooStruct { f, .. }: FooStruct,
) { }
"#,
        )
    }
}
