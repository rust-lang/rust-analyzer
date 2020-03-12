//! Assorted functions shared by several assists.
pub(crate) mod insert_use;

use hir::{
    AsAssocItem, AssocItemContainer, ModPath, Module, ModuleDef, PathResolution, Semantics, Trait,
    Type,
};
use ra_ide_db::{imports_locator::ImportsLocator, RootDatabase};
use ra_prof::profile;
use ra_syntax::{
    ast::{self, make, NameOwner},
    match_ast, AstNode, SyntaxNode, T,
};
use rustc_hash::FxHashSet;
use std::collections::BTreeSet;

pub use insert_use::insert_use_statement;

pub fn get_missing_impl_items(
    sema: &Semantics<RootDatabase>,
    impl_def: &ast::ImplDef,
) -> Vec<hir::AssocItem> {
    // Names must be unique between constants and functions. However, type aliases
    // may share the same name as a function or constant.
    let mut impl_fns_consts = FxHashSet::default();
    let mut impl_type = FxHashSet::default();

    if let Some(item_list) = impl_def.item_list() {
        for item in item_list.impl_items() {
            match item {
                ast::ImplItem::FnDef(f) => {
                    if let Some(n) = f.name() {
                        impl_fns_consts.insert(n.syntax().to_string());
                    }
                }

                ast::ImplItem::TypeAliasDef(t) => {
                    if let Some(n) = t.name() {
                        impl_type.insert(n.syntax().to_string());
                    }
                }

                ast::ImplItem::ConstDef(c) => {
                    if let Some(n) = c.name() {
                        impl_fns_consts.insert(n.syntax().to_string());
                    }
                }
            }
        }
    }

    resolve_target_trait(sema, impl_def).map_or(vec![], |target_trait| {
        target_trait
            .items(sema.db)
            .iter()
            .filter(|i| match i {
                hir::AssocItem::Function(f) => {
                    !impl_fns_consts.contains(&f.name(sema.db).to_string())
                }
                hir::AssocItem::TypeAlias(t) => !impl_type.contains(&t.name(sema.db).to_string()),
                hir::AssocItem::Const(c) => c
                    .name(sema.db)
                    .map(|n| !impl_fns_consts.contains(&n.to_string()))
                    .unwrap_or_default(),
            })
            .cloned()
            .collect()
    })
}

pub(crate) fn resolve_target_trait(
    sema: &Semantics<RootDatabase>,
    impl_def: &ast::ImplDef,
) -> Option<hir::Trait> {
    let ast_path = impl_def
        .target_trait()
        .map(|it| it.syntax().clone())
        .and_then(ast::PathType::cast)?
        .path()?;

    match sema.resolve_path(&ast_path) {
        Some(hir::PathResolution::Def(hir::ModuleDef::Trait(def))) => Some(def),
        _ => None,
    }
}

pub(crate) fn invert_boolean_expression(expr: ast::Expr) -> ast::Expr {
    if let Some(expr) = invert_special_case(&expr) {
        return expr;
    }
    make::expr_prefix(T![!], expr)
}

fn invert_special_case(expr: &ast::Expr) -> Option<ast::Expr> {
    match expr {
        ast::Expr::BinExpr(bin) => match bin.op_kind()? {
            ast::BinOp::NegatedEqualityTest => bin.replace_op(T![==]).map(|it| it.into()),
            ast::BinOp::EqualityTest => bin.replace_op(T![!=]).map(|it| it.into()),
            _ => None,
        },
        ast::Expr::PrefixExpr(pe) if pe.op_kind()? == ast::PrefixOp::Not => pe.expr(),
        // FIXME:
        // ast::Expr::Literal(true | false )
        _ => None,
    }
}

pub struct UnresolvedElement {
    pub import_candidate: ImportCandidate,
    pub module_with_element: Module,
    pub element_syntax: SyntaxNode,
}

impl UnresolvedElement {
    pub fn for_method_call(
        method_call: ast::MethodCallExpr,
        sema: &Semantics<RootDatabase>,
    ) -> Option<Self> {
        let syntax_under_caret = method_call.syntax().to_owned();
        let module_with_element = sema.scope(&syntax_under_caret).module()?;
        Some(Self {
            import_candidate: ImportCandidate::for_method_call(sema, &method_call)?,
            module_with_element,
            element_syntax: syntax_under_caret,
        })
    }

    pub fn for_path(path_under_caret: ast::Path, sema: &Semantics<RootDatabase>) -> Option<Self> {
        let syntax_under_caret = path_under_caret.syntax().to_owned();
        for node in syntax_under_caret.ancestors() {
            match_ast! {
                match node {
                    ast::UseItem(_it) => {
                        return None;
                    },
                    ast::Attr(_it) => {
                        return None;
                    },
                    _ => (),
                }
            }
        }

        let module_with_element = sema.scope(&syntax_under_caret).module()?;
        Some(Self {
            import_candidate: ImportCandidate::for_regular_path(sema, &path_under_caret)?,
            module_with_element,
            element_syntax: syntax_under_caret,
        })
    }

    fn get_search_query(&self) -> &str {
        match &self.import_candidate {
            ImportCandidate::UnqualifiedName(name) => name,
            ImportCandidate::QualifierStart(qualifier_start) => qualifier_start,
            ImportCandidate::TraitAssocItem(_, trait_assoc_item_name) => trait_assoc_item_name,
            ImportCandidate::TraitMethod(_, trait_method_name) => trait_method_name,
        }
    }

    pub(crate) fn get_import_group_message(&self) -> String {
        match &self.import_candidate {
            ImportCandidate::UnqualifiedName(name) => format!("Import {}", name),
            ImportCandidate::QualifierStart(qualifier_start) => {
                format!("Import {}", qualifier_start)
            }
            ImportCandidate::TraitAssocItem(_, trait_assoc_item_name) => {
                format!("Import a trait for item {}", trait_assoc_item_name)
            }
            ImportCandidate::TraitMethod(_, trait_method_name) => {
                format!("Import a trait for method {}", trait_method_name)
            }
        }
    }

    pub(crate) fn search_for_imports(&self, db: &RootDatabase) -> BTreeSet<ModPath> {
        let _p = profile("auto_import::search_for_imports");
        let current_crate = self.module_with_element.krate();
        ImportsLocator::new(db)
            .find_imports(&self.get_search_query())
            .into_iter()
            .filter_map(|module_def| match &self.import_candidate {
                ImportCandidate::TraitAssocItem(assoc_item_type, _) => {
                    let located_assoc_item = match module_def {
                        ModuleDef::Function(located_function) => located_function
                            .as_assoc_item(db)
                            .map(|assoc| assoc.container(db))
                            .and_then(Self::assoc_to_trait),
                        ModuleDef::Const(located_const) => located_const
                            .as_assoc_item(db)
                            .map(|assoc| assoc.container(db))
                            .and_then(Self::assoc_to_trait),
                        _ => None,
                    }?;

                    let mut trait_candidates = FxHashSet::default();
                    trait_candidates.insert(located_assoc_item.into());

                    assoc_item_type
                        .iterate_path_candidates(
                            db,
                            current_crate,
                            &trait_candidates,
                            None,
                            |_, assoc| Self::assoc_to_trait(assoc.container(db)),
                        )
                        .map(ModuleDef::from)
                }
                ImportCandidate::TraitMethod(function_callee, _) => {
                    let located_assoc_item =
                        if let ModuleDef::Function(located_function) = module_def {
                            located_function
                                .as_assoc_item(db)
                                .map(|assoc| assoc.container(db))
                                .and_then(Self::assoc_to_trait)
                        } else {
                            None
                        }?;

                    let mut trait_candidates = FxHashSet::default();
                    trait_candidates.insert(located_assoc_item.into());

                    function_callee
                        .iterate_method_candidates(
                            db,
                            current_crate,
                            &trait_candidates,
                            None,
                            |_, function| {
                                Self::assoc_to_trait(function.as_assoc_item(db)?.container(db))
                            },
                        )
                        .map(ModuleDef::from)
                }
                _ => Some(module_def),
            })
            .filter_map(|module_def| self.module_with_element.find_use_path(db, module_def))
            .filter(|use_path| !use_path.segments.is_empty())
            .take(20)
            .collect::<BTreeSet<_>>()
    }

    fn assoc_to_trait(assoc: AssocItemContainer) -> Option<Trait> {
        if let AssocItemContainer::Trait(extracted_trait) = assoc {
            Some(extracted_trait)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum ImportCandidate {
    /// Simple name like 'HashMap'
    UnqualifiedName(String),
    /// First part of the qualified name.
    /// For 'std::collections::HashMap', that will be 'std'.
    QualifierStart(String),
    /// A trait associated function (with no self parameter) or associated constant.
    /// For 'test_mod::TestEnum::test_function', `Type` is the `test_mod::TestEnum` expression type
    /// and `String` is the `test_function`
    TraitAssocItem(Type, String),
    /// A trait method with self parameter.
    /// For 'test_enum.test_method()', `Type` is the `test_enum` expression type
    /// and `String` is the `test_method`
    TraitMethod(Type, String),
}

impl ImportCandidate {
    fn for_method_call(
        sema: &Semantics<RootDatabase>,
        method_call: &ast::MethodCallExpr,
    ) -> Option<Self> {
        if sema.resolve_method_call(method_call).is_some() {
            return None;
        }
        Some(Self::TraitMethod(
            sema.type_of_expr(&method_call.expr()?)?,
            method_call.name_ref()?.syntax().to_string(),
        ))
    }

    fn for_regular_path(
        sema: &Semantics<RootDatabase>,
        path_under_caret: &ast::Path,
    ) -> Option<Self> {
        if sema.resolve_path(path_under_caret).is_some() {
            return None;
        }

        let segment = path_under_caret.segment()?;
        if let Some(qualifier) = path_under_caret.qualifier() {
            let qualifier_start = qualifier.syntax().descendants().find_map(ast::NameRef::cast)?;
            let qualifier_start_path =
                qualifier_start.syntax().ancestors().find_map(ast::Path::cast)?;
            if let Some(qualifier_start_resolution) = sema.resolve_path(&qualifier_start_path) {
                let qualifier_resolution = if qualifier_start_path == qualifier {
                    qualifier_start_resolution
                } else {
                    sema.resolve_path(&qualifier)?
                };
                if let PathResolution::Def(ModuleDef::Adt(assoc_item_path)) = qualifier_resolution {
                    Some(ImportCandidate::TraitAssocItem(
                        assoc_item_path.ty(sema.db),
                        segment.syntax().to_string(),
                    ))
                } else {
                    None
                }
            } else {
                Some(ImportCandidate::QualifierStart(qualifier_start.syntax().to_string()))
            }
        } else {
            Some(ImportCandidate::UnqualifiedName(
                segment.syntax().descendants().find_map(ast::NameRef::cast)?.syntax().to_string(),
            ))
        }
    }
}
