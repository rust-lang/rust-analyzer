//! Look up accessible paths for items.
use either::Either;
use hir::{AsAssocItem, AssocItemContainer, ModuleDef, PrefixKind, Semantics};
use rustc_hash::FxHashSet;
use syntax::{ast, AstNode};

use crate::{imports_locator, RootDatabase};

#[derive(Debug)]
pub enum ImportCandidate {
    // TODO kb docs
    Name(PathImportCandidate),

    // TODO kb collapse the trait enums?
    //
    /// A trait associated function (with no self parameter) or associated constant.
    /// For 'test_mod::TestEnum::test_function', `ty` is the `test_mod::TestEnum` expression type
    /// and `name` is the `test_function`
    TraitAssocItem(TraitImportCandidate),
    /// A trait method with self parameter.
    /// For 'test_enum.test_method()', `ty` is the `test_enum` expression type
    /// and `name` is the `test_method`
    TraitMethod(TraitImportCandidate),
}

#[derive(Debug)]
pub struct TraitImportCandidate {
    pub receiver_ty: hir::Type,
    pub name: NameToImport,
}

#[derive(Debug)]
pub struct PathImportCandidate {
    pub qualifier: Option<ast::Path>,
    pub name: NameToImport,
}

#[derive(Debug)]
pub enum NameToImport {
    Exact(String),
    Fuzzy(String),
}

impl NameToImport {
    pub fn text(&self) -> &str {
        match self {
            NameToImport::Exact(text) => text.as_str(),
            NameToImport::Fuzzy(text) => text.as_str(),
        }
    }
}

#[derive(Debug)]
pub struct ImportAssets {
    import_candidate: ImportCandidate,
    module_with_candidate: hir::Module,
}

impl ImportAssets {
    pub fn for_method_call(
        method_call: ast::MethodCallExpr,
        sema: &Semantics<RootDatabase>,
    ) -> Option<Self> {
        let syntax_under_caret = method_call.syntax().to_owned();
        let module_with_candidate = sema.scope(&syntax_under_caret).module()?;
        Some(Self {
            import_candidate: ImportCandidate::for_method_call(sema, &method_call)?,
            module_with_candidate,
        })
    }

    pub fn for_exact_path(
        fully_qualified_path: ast::Path,
        sema: &Semantics<RootDatabase>,
    ) -> Option<Self> {
        let syntax_under_caret = fully_qualified_path.syntax().to_owned();
        if syntax_under_caret.ancestors().find_map(ast::Use::cast).is_some() {
            return None;
        }

        let module_with_candidate = sema.scope(&syntax_under_caret).module()?;
        Some(Self {
            import_candidate: ImportCandidate::for_regular_path(sema, &fully_qualified_path)?,
            module_with_candidate,
        })
    }
}

impl ImportAssets {
    pub fn import_candidate(&self) -> &ImportCandidate {
        &self.import_candidate
    }

    fn name_to_import(&self) -> &NameToImport {
        match &self.import_candidate {
            ImportCandidate::Name(candidate) => &candidate.name,
            ImportCandidate::TraitAssocItem(candidate)
            | ImportCandidate::TraitMethod(candidate) => &candidate.name,
        }
    }

    pub fn search_for_imports(
        &self,
        sema: &Semantics<RootDatabase>,
        prefix_kind: PrefixKind,
    ) -> Vec<(hir::ModPath, hir::ItemInNs)> {
        let _p = profile::span("import_assets::search_for_imports");
        self.search_for(sema, Some(prefix_kind))
    }

    /// This may return non-absolute paths if a part of the returned path is already imported into scope.
    pub fn search_for_relative_paths(
        &self,
        sema: &Semantics<RootDatabase>,
    ) -> Vec<(hir::ModPath, hir::ItemInNs)> {
        let _p = profile::span("import_assets::search_for_relative_paths");
        self.search_for(sema, None)
    }

    fn search_for(
        &self,
        sema: &Semantics<RootDatabase>,
        prefixed: Option<hir::PrefixKind>,
    ) -> Vec<(hir::ModPath, hir::ItemInNs)> {
        let db = sema.db;
        let mut trait_candidates = FxHashSet::default();
        let current_crate = self.module_with_candidate.krate();

        let filter = |candidate: Either<hir::ModuleDef, hir::MacroDef>| {
            trait_candidates.clear();
            match &self.import_candidate {
                ImportCandidate::TraitAssocItem(trait_candidate) => {
                    let located_assoc_item = match candidate {
                        Either::Left(ModuleDef::Function(located_function)) => {
                            located_function.as_assoc_item(db)
                        }
                        Either::Left(ModuleDef::Const(located_const)) => {
                            located_const.as_assoc_item(db)
                        }
                        _ => None,
                    }
                    .map(|assoc| assoc.container(db))
                    .and_then(Self::assoc_to_trait)?;

                    trait_candidates.insert(located_assoc_item.into());

                    trait_candidate
                        .receiver_ty
                        .iterate_path_candidates(
                            db,
                            current_crate,
                            &trait_candidates,
                            None,
                            |_, assoc| Self::assoc_to_trait(assoc.container(db)),
                        )
                        .map(ModuleDef::from)
                        .map(Either::Left)
                }
                ImportCandidate::TraitMethod(trait_candidate) => {
                    let located_assoc_item =
                        if let Either::Left(ModuleDef::Function(located_function)) = candidate {
                            located_function
                                .as_assoc_item(db)
                                .map(|assoc| assoc.container(db))
                                .and_then(Self::assoc_to_trait)
                        } else {
                            None
                        }?;

                    trait_candidates.insert(located_assoc_item.into());

                    trait_candidate
                        .receiver_ty
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
                        .map(Either::Left)
                }
                _ => Some(candidate),
            }
        };

        // TODO kb useless collects due to different opaque types + stupid clone()'s
        let unfiltered_imports = match self.name_to_import() {
            NameToImport::Exact(exact_name) => {
                imports_locator::find_exact_imports(sema, current_crate, exact_name.clone())
                    .collect::<Vec<_>>()
            }
            NameToImport::Fuzzy(fuzzy_name) => match self.import_candidate {
                ImportCandidate::TraitAssocItem(_) | ImportCandidate::TraitMethod(_) => {
                    imports_locator::find_similar_associated_items(
                        sema,
                        current_crate,
                        fuzzy_name.clone(),
                    )
                    .collect::<Vec<_>>()
                }
                _ => imports_locator::find_similar_imports(
                    sema,
                    current_crate,
                    fuzzy_name.clone(),
                    true,
                )
                .collect::<Vec<_>>(),
            },
        }
        .into_iter();

        let mut res = unfiltered_imports
            .filter_map(filter)
            .filter_map(|candidate| {
                let item: hir::ItemInNs = candidate.either(Into::into, Into::into);
                if let Some(prefix_kind) = prefixed {
                    self.module_with_candidate.find_use_path_prefixed(db, item, prefix_kind)
                } else {
                    self.module_with_candidate.find_use_path(db, item)
                }
                .map(|path| (path, item))
            })
            .filter(|(use_path, _)| use_path.len() > 1)
            .take(20)
            .collect::<Vec<_>>();
        res.sort_by_key(|(path, _)| path.clone());
        res
    }

    fn assoc_to_trait(assoc: AssocItemContainer) -> Option<hir::Trait> {
        if let AssocItemContainer::Trait(extracted_trait) = assoc {
            Some(extracted_trait)
        } else {
            None
        }
    }
}

impl ImportCandidate {
    fn for_method_call(
        sema: &Semantics<RootDatabase>,
        method_call: &ast::MethodCallExpr,
    ) -> Option<Self> {
        match sema.resolve_method_call(method_call) {
            Some(_) => None,
            None => Some(Self::TraitMethod(TraitImportCandidate {
                receiver_ty: sema.type_of_expr(&method_call.receiver()?)?,
                name: NameToImport::Exact(method_call.name_ref()?.to_string()),
            })),
        }
    }

    fn for_regular_path(sema: &Semantics<RootDatabase>, path: &ast::Path) -> Option<Self> {
        if sema.resolve_path(path).is_some() {
            return None;
        }

        let segment = path.segment()?;
        let candidate = if let Some(qualifier) = path.qualifier() {
            let qualifier_start = qualifier.syntax().descendants().find_map(ast::NameRef::cast)?;
            let qualifier_start_path =
                qualifier_start.syntax().ancestors().find_map(ast::Path::cast)?;
            if let Some(qualifier_start_resolution) = sema.resolve_path(&qualifier_start_path) {
                let qualifier_resolution = if qualifier_start_path == qualifier {
                    qualifier_start_resolution
                } else {
                    sema.resolve_path(&qualifier)?
                };
                match qualifier_resolution {
                    hir::PathResolution::Def(hir::ModuleDef::Adt(assoc_item_path)) => {
                        ImportCandidate::TraitAssocItem(TraitImportCandidate {
                            receiver_ty: assoc_item_path.ty(sema.db),
                            name: NameToImport::Exact(segment.name_ref()?.to_string()),
                        })
                    }
                    _ => return None,
                }
            } else {
                ImportCandidate::Name(PathImportCandidate {
                    qualifier: Some(qualifier),
                    name: NameToImport::Exact(qualifier_start.to_string()),
                })
            }
        } else {
            ImportCandidate::Name(PathImportCandidate {
                qualifier: None,
                name: NameToImport::Exact(
                    segment.syntax().descendants().find_map(ast::NameRef::cast)?.to_string(),
                ),
            })
        };
        Some(candidate)
    }
}
