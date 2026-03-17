use std::iter;

use either::Either;
use hir::{HasCrate, Module, ModuleDef, Name, Variant};
use ide_db::{
    FxHashSet, RootDatabase,
    defs::Definition,
    helpers::mod_path_to_ast,
    imports::insert_use::{ImportScope, ImportScopeKind, InsertUseConfig, insert_use_with_editor},
    path_transform::PathTransform,
    search::FileReference,
};
use itertools::Itertools;
use syntax::{
    Direction, Edition, SyntaxElement,
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
    ast::{
        self, AstNode, HasAttrs, HasGenericParams, HasName, HasVisibility,
        edit::{AstNodeEdit, IndentLevel},
        make,
        syntax_factory::SyntaxFactory,
    },
    match_ast,
    syntax_editor::{Position, SyntaxEditor},
};

use crate::{AssistContext, AssistId, Assists};

// Assist: extract_struct_from_enum_variant
//
// Extracts a struct from enum variant.
//
// ```
// enum A { $0One(u32, u32) }
// ```
// ->
// ```
// struct One(u32, u32);
//
// enum A { One(One) }
// ```
pub(crate) fn extract_struct_from_enum_variant(
    acc: &mut Assists,
    ctx: &AssistContext<'_>,
) -> Option<()> {
    let variant = ctx.find_node_at_offset::<ast::Variant>()?;
    let field_list = extract_field_list_if_applicable(&variant)?;

    let variant_name = variant.name()?;
    let variant_hir = ctx.sema.to_def(&variant)?;
    if existing_definition(ctx.db(), &variant_name, &variant_hir) {
        cov_mark::hit!(test_extract_enum_not_applicable_if_struct_exists);
        return None;
    }

    let enum_ast = variant.parent_enum();
    let enum_hir = ctx.sema.to_def(&enum_ast)?;
    let target = variant.syntax().text_range();
    acc.add(
        AssistId::refactor_rewrite("extract_struct_from_enum_variant"),
        "Extract struct from enum variant",
        target,
        |builder| {
            let edition = enum_hir.krate(ctx.db()).edition(ctx.db());
            let variant_hir_name = variant_hir.name(ctx.db());
            let enum_module_def = ModuleDef::from(enum_hir);
            let usages = Definition::Variant(variant_hir).usages(&ctx.sema).all();

            let mut visited_modules_set = FxHashSet::default();
            let current_module = enum_hir.module(ctx.db());
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
                    &enum_module_def,
                    &variant_hir_name,
                    references,
                );
                let Some((segment, ..)) = processed.first() else { continue };
                let mut editor = builder.make_editor(segment.syntax());
                let factory = SyntaxFactory::with_mappings();
                processed.into_iter().for_each(|(path, node, import)| {
                    apply_references(
                        ctx.config.insert_use,
                        path,
                        node,
                        import,
                        edition,
                        &mut editor,
                        &factory,
                    )
                });
                editor.add_mappings(factory.finish_with_mappings());
                builder.add_file_edits(file_id.file_id(ctx.db()), editor);
            }
            let mut editor = builder.make_editor(variant.syntax());
            let factory = SyntaxFactory::with_mappings();
            if let Some(references) = def_file_references {
                let processed = process_references(
                    ctx,
                    &mut visited_modules_set,
                    &enum_module_def,
                    &variant_hir_name,
                    references,
                );
                processed.into_iter().for_each(|(path, node, import)| {
                    apply_references(
                        ctx.config.insert_use,
                        path,
                        node,
                        import,
                        edition,
                        &mut editor,
                        &factory,
                    )
                });
            }

            let generic_params = enum_ast
                .generic_param_list()
                .and_then(|known_generics| extract_generic_params(&known_generics, &field_list));
            let generics = generic_params.clone();

            // resolve GenericArg in field_list to actual type
            let field_list = if let Some((target_scope, source_scope)) =
                ctx.sema.scope(enum_ast.syntax()).zip(ctx.sema.scope(field_list.syntax()))
            {
                let field_list = field_list.reset_indent();
                let field_list =
                    PathTransform::generic_transformation(&target_scope, &source_scope)
                        .apply(field_list.syntax());
                match_ast! {
                    match field_list {
                        ast::RecordFieldList(field_list) => Either::Left(field_list),
                        ast::TupleFieldList(field_list) => Either::Right(field_list),
                        _ => unreachable!(),
                    }
                }
            } else {
                match &field_list {
                    Either::Left(field_list) => Either::Left(field_list.clone_subtree()),
                    Either::Right(field_list) => Either::Right(field_list.clone_subtree()),
                }
            };
            let (comments_to_insert, comments_to_remove) =
                collect_comments_to_move(variant.syntax(), &factory);

            let def = create_struct_def(
                variant_name.clone(),
                &field_list,
                generics,
                &enum_ast,
                comments_to_insert,
                &factory,
            );

            let enum_ast = variant.parent_enum();
            let indent = enum_ast.indent_level();
            let def = def.indent(indent);

            editor.insert_all(
                Position::before(enum_ast.syntax()),
                vec![
                    def.syntax().clone().into(),
                    factory.whitespace(&format!("\n\n{indent}")).into(),
                ],
            );

            comments_to_remove.into_iter().for_each(|elem| editor.delete(elem));
            update_variant(&mut editor, &factory, &variant, generic_params);

            editor.add_mappings(factory.finish_with_mappings());
            builder.add_file_edits(ctx.vfs_file_id(), editor);
        },
    )
}

fn extract_field_list_if_applicable(
    variant: &ast::Variant,
) -> Option<Either<ast::RecordFieldList, ast::TupleFieldList>> {
    match variant.kind() {
        ast::StructKind::Record(field_list) if field_list.fields().next().is_some() => {
            Some(Either::Left(field_list))
        }
        ast::StructKind::Tuple(field_list) if field_list.fields().count() > 1 => {
            Some(Either::Right(field_list))
        }
        _ => None,
    }
}

fn existing_definition(db: &RootDatabase, variant_name: &ast::Name, variant: &Variant) -> bool {
    variant
        .parent_enum(db)
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

fn extract_generic_params(
    known_generics: &ast::GenericParamList,
    field_list: &Either<ast::RecordFieldList, ast::TupleFieldList>,
) -> Option<ast::GenericParamList> {
    let mut generics = known_generics.generic_params().map(|param| (param, false)).collect_vec();

    let tagged_one = match field_list {
        Either::Left(field_list) => field_list
            .fields()
            .filter_map(|f| f.ty())
            .fold(false, |tagged, ty| tag_generics_in_variant(&ty, &mut generics) || tagged),
        Either::Right(field_list) => field_list
            .fields()
            .filter_map(|f| f.ty())
            .fold(false, |tagged, ty| tag_generics_in_variant(&ty, &mut generics) || tagged),
    };

    let generics = generics.into_iter().filter_map(|(param, tag)| tag.then_some(param));
    tagged_one.then(|| make::generic_param_list(generics))
}

fn tag_generics_in_variant(ty: &ast::Type, generics: &mut [(ast::GenericParam, bool)]) -> bool {
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

fn create_struct_def(
    name: ast::Name,
    field_list: &Either<ast::RecordFieldList, ast::TupleFieldList>,
    generics: Option<ast::GenericParamList>,
    enum_: &ast::Enum,
    comments_to_move: Vec<SyntaxElement>,
    factory: &SyntaxFactory,
) -> ast::Struct {
    let enum_vis = enum_.visibility();
    let field_list: ast::FieldList = match field_list {
        Either::Left(field_list) => field_list.clone_subtree().into(),
        Either::Right(field_list) => field_list.clone_subtree().into(),
    };
    let field_list = field_list.indent(IndentLevel::single());

    let strukt = make::struct_(enum_vis.clone(), name, generics, field_list);
    let mut struct_editor = SyntaxEditor::new(strukt.syntax().clone());

    // for fields without any existing visibility, use visibility of enum
    if let Some(vis) = &enum_vis
        && let Some(field_list) = strukt.field_list()
    {
        match field_list {
            ast::FieldList::RecordFieldList(field_list) => {
                field_list
                    .fields()
                    .filter(|field| field.visibility().is_none())
                    .filter_map(|field| field.name())
                    .for_each(|it| {
                        struct_editor.insert_all(
                            Position::before(it.syntax()),
                            vec![
                                vis.syntax().clone_subtree().clone_for_update().into(),
                                factory.whitespace(" ").into(),
                            ],
                        )
                    });
            }
            ast::FieldList::TupleFieldList(field_list) => {
                field_list
                    .fields()
                    .filter(|field| field.visibility().is_none())
                    .filter_map(|field| field.ty())
                    .for_each(|it| {
                        struct_editor.insert_all(
                            Position::before(it.syntax()),
                            vec![
                                vis.syntax().clone_subtree().clone_for_update().into(),
                                factory.whitespace(" ").into(),
                            ],
                        )
                    });
            }
        }
    }

    // take comments from variant
    struct_editor.insert_all(Position::first_child_of(strukt.syntax()), comments_to_move);

    // copy attributes from enum
    struct_editor.insert_all(
        Position::first_child_of(strukt.syntax()),
        enum_
            .attrs()
            .flat_map(|it| {
                vec![
                    it.syntax().clone_subtree().clone_for_update().into(),
                    factory.whitespace("\n").into(),
                ]
            })
            .collect(),
    );

    let struct_edit = struct_editor.finish();
    ast::Struct::cast(struct_edit.new_root().clone()).expect("struct root should stay a struct")
}

fn update_variant(
    editor: &mut SyntaxEditor,
    factory: &SyntaxFactory,
    variant: &ast::Variant,
    generics: Option<ast::GenericParamList>,
) -> Option<()> {
    let name = variant.name()?;
    let generic_args = generics
        .filter(|generics| generics.generic_params().count() > 0)
        .map(|generics| generics.to_generic_args());
    let ty = match generic_args {
        Some(generic_args) => factory.ty(&format!("{name}{generic_args}")),
        None => factory.ty(&name.text()),
    };

    // change from a record to a tuple field list
    let tuple_field = factory.tuple_field(None, ty);
    let field_list = factory.tuple_field_list(iter::once(tuple_field));
    editor.replace(variant.field_list()?.syntax(), field_list.syntax());

    // remove any ws after the name
    if let Some(ws) = name
        .syntax()
        .siblings_with_tokens(Direction::Next)
        .find_map(|tok| tok.into_token().filter(|tok| tok.kind() == WHITESPACE))
    {
        editor.delete(SyntaxElement::Token(ws));
    }

    Some(())
}

fn collect_comments_to_move(
    node: &SyntaxNode,
    factory: &SyntaxFactory,
) -> (Vec<SyntaxElement>, Vec<SyntaxElement>) {
    let mut remove_next_ws = false;
    let mut comments_to_insert = Vec::new();
    let mut comments_to_remove = Vec::new();

    node.children_with_tokens().for_each(|child| match child.kind() {
        COMMENT => {
            remove_next_ws = true;
            let comment = child
                .clone()
                .into_token()
                .map(|token| make_comment_token(token.text()))
                .unwrap_or(child.clone());
            comments_to_insert.push(comment);
            comments_to_remove.push(child);
        }
        WHITESPACE if remove_next_ws => {
            remove_next_ws = false;
            comments_to_insert.push(factory.whitespace("\n").into());
            comments_to_remove.push(child);
        }
        _ => remove_next_ws = false,
    });

    (comments_to_insert, comments_to_remove)
}

fn make_comment_token(text: &str) -> SyntaxElement {
    let parse = syntax::SourceFile::parse(text, Edition::CURRENT);
    let comment = parse.tree().syntax().clone_for_update().first_child_or_token().unwrap();
    if let Some(token) = comment.as_token() {
        token.detach();
    }
    comment
}

fn apply_references(
    insert_use_cfg: InsertUseConfig,
    segment: ast::PathSegment,
    node: SyntaxNode,
    import: Option<(ImportScope, hir::ModPath)>,
    edition: Edition,
    editor: &mut SyntaxEditor,
    factory: &SyntaxFactory,
) {
    if let Some((scope, path)) = import {
        let needs_newline_fix = needs_module_import_newline_fix(&scope);
        insert_use_with_editor(
            &scope,
            mod_path_to_ast(&path, edition),
            &insert_use_cfg,
            editor,
            factory,
        );
        if needs_newline_fix && let Some(l_curly) = scope_l_curly_token(&scope) {
            let indent = scope_indent_after_l_curly(&scope);
            // `insert_use_with_editor` inserts newline then `use` at the same position in this
            // branch; insertion order makes the `use` appear before the newline.
            if !indent.is_empty() {
                editor.insert(Position::after(l_curly.clone()), factory.whitespace(&indent));
            }
            editor.insert(Position::after(l_curly), factory.whitespace("\n"));
        }
    }
    let path = factory.path_from_segments(iter::once(segment.clone_subtree()), false);
    editor.insert_all(
        Position::before(segment.syntax()),
        vec![path.syntax().clone().into(), factory.token(T!['(']).into()],
    );
    editor.insert(Position::after(&node), factory.token(T![')']));
}

fn process_references(
    ctx: &AssistContext<'_>,
    visited_modules: &mut FxHashSet<Module>,
    enum_module_def: &ModuleDef,
    variant_hir_name: &Name,
    refs: Vec<FileReference>,
) -> Vec<(ast::PathSegment, SyntaxNode, Option<(ImportScope, hir::ModPath)>)> {
    // we have to recollect here eagerly as we are about to edit the tree we need to calculate the changes
    // and corresponding nodes up front
    refs.into_iter()
        .flat_map(|reference| {
            let (segment, scope_node, module) = reference_to_node(&ctx.sema, reference)?;
            if !visited_modules.contains(&module) {
                let cfg =
                    ctx.config.find_path_config(ctx.sema.is_nightly(module.krate(ctx.sema.db)));
                let mod_path = module.find_use_path(
                    ctx.sema.db,
                    *enum_module_def,
                    ctx.config.insert_use.prefix_kind,
                    cfg,
                );
                if let Some(mut mod_path) = mod_path {
                    mod_path.pop_segment();
                    mod_path.push_segment(variant_hir_name.clone());
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
    let segment =
        reference.name.as_name_ref()?.syntax().parent().and_then(ast::PathSegment::cast)?;

    // filter out the reference in marco
    let segment_range = segment.syntax().text_range();
    if segment_range != reference.range {
        return None;
    }

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
    Some((segment, expr_or_pat, module))
}

fn scope_l_curly_token(scope: &ImportScope) -> Option<SyntaxToken> {
    match &scope.kind {
        ImportScopeKind::File(_) => None,
        ImportScopeKind::Module(item_list) => item_list.l_curly_token(),
        ImportScopeKind::Block(stmt_list) => stmt_list.l_curly_token(),
    }
}

fn needs_module_import_newline_fix(scope: &ImportScope) -> bool {
    let Some(_l_curly) = scope_l_curly_token(scope) else { return false };
    if scope.as_syntax_node().children().any(|child| ast::Use::can_cast(child.kind())) {
        return false;
    }

    let has_leading_inner_element = scope
        .as_syntax_node()
        .children_with_tokens()
        .skip(1)
        .take_while(|child| match child {
            SyntaxElement::Node(node) => {
                ast::AnyHasAttrs::cast(node.clone()).is_some_and(|has_attrs| {
                    has_attrs
                        .attrs()
                        .any(|attr| attr.excl_token().is_some() && attr.l_brack_token().is_some())
                        && ast::Item::cast(node.clone()).is_none()
                })
            }
            SyntaxElement::Token(token) => [WHITESPACE, COMMENT, SHEBANG].contains(&token.kind()),
        })
        .filter(|child| child.as_token().is_none_or(|t| t.kind() != WHITESPACE))
        .last()
        .is_some();

    !has_leading_inner_element
}

fn scope_indent_after_l_curly(scope: &ImportScope) -> String {
    let Some(l_curly) = scope_l_curly_token(scope) else { return String::new() };
    let Some(token) = l_curly.next_sibling_or_token().and_then(|it| it.into_token()) else {
        return String::new();
    };
    if token.kind() != WHITESPACE {
        return String::new();
    }

    token.text().rsplit('\n').next().unwrap_or_default().to_owned()
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn test_with_marco() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
macro_rules! foo {
    ($x:expr) => {
        $x
    };
}

enum TheEnum {
    TheVariant$0 { the_field: u8 },
}

fn main() {
    foo![TheEnum::TheVariant { the_field: 42 }];
}
"#,
            r#"
macro_rules! foo {
    ($x:expr) => {
        $x
    };
}

struct TheVariant { the_field: u8 }

enum TheEnum {
    TheVariant(TheVariant),
}

fn main() {
    foo![TheEnum::TheVariant { the_field: 42 }];
}
"#,
        );
    }

    #[test]
    fn issue_16197() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum Foo {
    Bar $0{ node: Box<Self> },
    Nil,
}
"#,
            r#"
struct Bar { node: Box<Foo> }

enum Foo {
    Bar(Bar),
    Nil,
}
"#,
        );
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum Foo {
    Bar $0{ node: Box<Self>, a: Arc<Box<Self>> },
    Nil,
}
"#,
            r#"
struct Bar { node: Box<Foo>, a: Arc<Box<Foo>> }

enum Foo {
    Bar(Bar),
    Nil,
}
"#,
        );
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum Foo {
    Nil(Box$0<Self>, Arc<Box<Self>>),
}
"#,
            r#"
struct Nil(Box<Foo>, Arc<Box<Foo>>);

enum Foo {
    Nil(Nil),
}
"#,
        );
    }

    #[test]
    fn test_extract_struct_several_fields_tuple() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One(u32, u32) }",
            r#"struct One(u32, u32);

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_several_fields_named() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One { foo: u32, bar: u32 } }",
            r#"struct One { foo: u32, bar: u32 }

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_one_field_named() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One { foo: u32 } }",
            r#"struct One { foo: u32 }

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_carries_over_generics() {
        check_assist(
            extract_struct_from_enum_variant,
            r"enum En<T> { Var { a: T$0 } }",
            r#"struct Var<T> { a: T }

enum En<T> { Var(Var<T>) }"#,
        );
    }

    #[test]
    fn test_extract_struct_carries_over_attributes() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
#[derive(Debug)]
#[derive(Clone)]
enum Enum { Variant{ field: u32$0 } }"#,
            r#"
#[derive(Debug)]
#[derive(Clone)]
struct Variant { field: u32 }

#[derive(Debug)]
#[derive(Clone)]
enum Enum { Variant(Variant) }"#,
        );
    }

    #[test]
    fn test_extract_struct_indent_to_parent_enum() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum Enum {
    Variant {
        field: u32$0
    }
}"#,
            r#"
struct Variant {
    field: u32
}

enum Enum {
    Variant(Variant)
}"#,
        );
    }

    #[test]
    fn test_extract_struct_indent_to_parent_enum_in_mod() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
mod indenting {
    enum Enum {
        Variant {
            field: u32$0
        }
    }
}"#,
            r#"
mod indenting {
    struct Variant {
        field: u32
    }

    enum Enum {
        Variant(Variant)
    }
}"#,
        );
    }

    #[test]
    fn test_extract_struct_keep_comments_and_attrs_one_field_named() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum A {
    $0One {
        // leading comment
        /// doc comment
        #[an_attr]
        foo: u32
        // trailing comment
    }
}"#,
            r#"
struct One {
    // leading comment
    /// doc comment
    #[an_attr]
    foo: u32
    // trailing comment
}

enum A {
    One(One)
}"#,
        );
    }

    #[test]
    fn test_extract_struct_keep_comments_and_attrs_several_fields_named() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum A {
    $0One {
        // comment
        /// doc
        #[attr]
        foo: u32,
        // comment
        #[attr]
        /// doc
        bar: u32
    }
}"#,
            r#"
struct One {
    // comment
    /// doc
    #[attr]
    foo: u32,
    // comment
    #[attr]
    /// doc
    bar: u32
}

enum A {
    One(One)
}"#,
        );
    }

    #[test]
    fn test_extract_struct_keep_comments_and_attrs_several_fields_tuple() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One(/* comment */ #[attr] u32, /* another */ u32 /* tail */) }",
            r#"
struct One(/* comment */ #[attr] u32, /* another */ u32 /* tail */);

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_move_struct_variant_comments() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum A {
    /* comment */
    // other
    /// comment
    #[attr]
    $0One {
        a: u32
    }
}"#,
            r#"
/* comment */
// other
/// comment
struct One {
    a: u32
}

enum A {
    #[attr]
    One(One)
}"#,
        );
    }

    #[test]
    fn test_extract_struct_move_tuple_variant_comments() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum A {
    /* comment */
    // other
    /// comment
    #[attr]
    $0One(u32, u32)
}"#,
            r#"
/* comment */
// other
/// comment
struct One(u32, u32);

enum A {
    #[attr]
    One(One)
}"#,
        );
    }

    #[test]
    fn test_extract_struct_keep_existing_visibility_named() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One{ a: u32, pub(crate) b: u32, pub(super) c: u32, d: u32 } }",
            r#"
struct One { a: u32, pub(crate) b: u32, pub(super) c: u32, d: u32 }

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_keep_existing_visibility_tuple() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One(u32, pub(crate) u32, pub(super) u32, u32) }",
            r#"
struct One(u32, pub(crate) u32, pub(super) u32, u32);

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_enum_variant_name_value_namespace() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"const One: () = ();
enum A { $0One(u32, u32) }"#,
            r#"const One: () = ();
struct One(u32, u32);

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_no_visibility() {
        check_assist(
            extract_struct_from_enum_variant,
            "enum A { $0One(u32, u32) }",
            r#"
struct One(u32, u32);

enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_pub_visibility() {
        check_assist(
            extract_struct_from_enum_variant,
            "pub enum A { $0One(u32, u32) }",
            r#"
pub struct One(pub u32, pub u32);

pub enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_pub_in_mod_visibility() {
        check_assist(
            extract_struct_from_enum_variant,
            "pub(in something) enum A { $0One{ a: u32, b: u32 } }",
            r#"
pub(in something) struct One { pub(in something) a: u32, pub(in something) b: u32 }

pub(in something) enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_pub_crate_visibility() {
        check_assist(
            extract_struct_from_enum_variant,
            "pub(crate) enum A { $0One{ a: u32, b: u32, c: u32 } }",
            r#"
pub(crate) struct One { pub(crate) a: u32, pub(crate) b: u32, pub(crate) c: u32 }

pub(crate) enum A { One(One) }"#,
        );
    }

    #[test]
    fn test_extract_struct_with_complex_imports() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"mod my_mod {
    fn another_fn() {
        let m = my_other_mod::MyEnum::MyField(1, 1);
    }

    pub mod my_other_mod {
        fn another_fn() {
            let m = MyEnum::MyField(1, 1);
        }

        pub enum MyEnum {
            $0MyField(u8, u8),
        }
    }
}

fn another_fn() {
    let m = my_mod::my_other_mod::MyEnum::MyField(1, 1);
}"#,
            r#"use my_mod::my_other_mod::MyField;

mod my_mod {
    use my_other_mod::MyField;

    fn another_fn() {
        let m = my_other_mod::MyEnum::MyField(MyField(1, 1));
    }

    pub mod my_other_mod {
        fn another_fn() {
            let m = MyEnum::MyField(MyField(1, 1));
        }

        pub struct MyField(pub u8, pub u8);

        pub enum MyEnum {
            MyField(MyField),
        }
    }
}

fn another_fn() {
    let m = my_mod::my_other_mod::MyEnum::MyField(MyField(1, 1));
}"#,
        );
    }

    #[test]
    fn extract_record_fix_references() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum E {
    $0V { i: i32, j: i32 }
}

fn f() {
    let E::V { i, j } = E::V { i: 9, j: 2 };
}
"#,
            r#"
struct V { i: i32, j: i32 }

enum E {
    V(V)
}

fn f() {
    let E::V(V { i, j }) = E::V(V { i: 9, j: 2 });
}
"#,
        )
    }

    #[test]
    fn extract_record_fix_references2() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum E {
    $0V(i32, i32)
}

fn f() {
    let E::V(i, j) = E::V(9, 2);
}
"#,
            r#"
struct V(i32, i32);

enum E {
    V(V)
}

fn f() {
    let E::V(V(i, j)) = E::V(V(9, 2));
}
"#,
        )
    }

    #[test]
    fn extract_fix_references_with_turbofish() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum E<T> {
    $0V(T, T)
}

fn f() {
    let _ = E::<i32>::V(9, 2);
}
"#,
            r#"
struct V<T>(T, T);

enum E<T> {
    V(V<T>)
}

fn f() {
    let _ = E::<i32>::V(V(9, 2));
}
"#,
        )
    }

    #[test]
    fn test_several_files() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
//- /main.rs
enum E {
    $0V(i32, i32)
}
mod foo;

//- /foo.rs
use crate::E;
fn f() {
    let e = E::V(9, 2);
}
"#,
            r#"
//- /main.rs
struct V(i32, i32);

enum E {
    V(V)
}
mod foo;

//- /foo.rs
use crate::{E, V};
fn f() {
    let e = E::V(V(9, 2));
}
"#,
        )
    }

    #[test]
    fn test_several_files_record() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
//- /main.rs
enum E {
    $0V { i: i32, j: i32 }
}
mod foo;

//- /foo.rs
use crate::E;
fn f() {
    let e = E::V { i: 9, j: 2 };
}
"#,
            r#"
//- /main.rs
struct V { i: i32, j: i32 }

enum E {
    V(V)
}
mod foo;

//- /foo.rs
use crate::{E, V};
fn f() {
    let e = E::V(V { i: 9, j: 2 });
}
"#,
        )
    }

    #[test]
    fn test_extract_struct_record_nested_call_exp() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum A { $0One { a: u32, b: u32 } }

struct B(A);

fn foo() {
    let _ = B(A::One { a: 1, b: 2 });
}
"#,
            r#"
struct One { a: u32, b: u32 }

enum A { One(One) }

struct B(A);

fn foo() {
    let _ = B(A::One(One { a: 1, b: 2 }));
}
"#,
        );
    }

    #[test]
    fn test_extract_enum_not_applicable_for_element_with_no_fields() {
        check_assist_not_applicable(extract_struct_from_enum_variant, r#"enum A { $0One }"#);
    }

    #[test]
    fn test_extract_enum_not_applicable_if_struct_exists() {
        cov_mark::check!(test_extract_enum_not_applicable_if_struct_exists);
        check_assist_not_applicable(
            extract_struct_from_enum_variant,
            r#"
struct One;
enum A { $0One(u8, u32) }
"#,
        );
    }

    #[test]
    fn test_extract_not_applicable_one_field() {
        check_assist_not_applicable(extract_struct_from_enum_variant, r"enum A { $0One(u32) }");
    }

    #[test]
    fn test_extract_not_applicable_no_field_tuple() {
        check_assist_not_applicable(extract_struct_from_enum_variant, r"enum A { $0None() }");
    }

    #[test]
    fn test_extract_not_applicable_no_field_named() {
        check_assist_not_applicable(extract_struct_from_enum_variant, r"enum A { $0None {} }");
    }

    #[test]
    fn test_extract_struct_only_copies_needed_generics() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum X<'a, 'b, 'x> {
    $0A { a: &'a &'x mut () },
    B { b: &'b () },
    C { c: () },
}
"#,
            r#"
struct A<'a, 'x> { a: &'a &'x mut () }

enum X<'a, 'b, 'x> {
    A(A<'a, 'x>),
    B { b: &'b () },
    C { c: () },
}
"#,
        );
    }

    #[test]
    fn test_extract_struct_with_lifetime_type_const() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum X<'b, T, V, const C: usize> {
    $0A { a: T, b: X<'b>, c: [u8; C] },
    D { d: V },
}
"#,
            r#"
struct A<'b, T, const C: usize> { a: T, b: X<'b>, c: [u8; C] }

enum X<'b, T, V, const C: usize> {
    A(A<'b, T, C>),
    D { d: V },
}
"#,
        );
    }

    #[test]
    fn test_extract_struct_without_generics() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum X<'a, 'b> {
    A { a: &'a () },
    B { b: &'b () },
    $0C { c: () },
}
"#,
            r#"
struct C { c: () }

enum X<'a, 'b> {
    A { a: &'a () },
    B { b: &'b () },
    C(C),
}
"#,
        );
    }

    #[test]
    fn test_extract_struct_keeps_trait_bounds() {
        check_assist(
            extract_struct_from_enum_variant,
            r#"
enum En<T: TraitT, V: TraitV> {
    $0A { a: T },
    B { b: V },
}
"#,
            r#"
struct A<T: TraitT> { a: T }

enum En<T: TraitT, V: TraitV> {
    A(A<T>),
    B { b: V },
}
"#,
        );
    }
}
