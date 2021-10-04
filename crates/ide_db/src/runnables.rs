use std::sync::Arc;

use base_db::{FileId, SourceDatabaseExt, SourceRoot, Upcast, salsa};
use either::Either;
use hir::{Crate, HasAttrs, HasSource, Semantics, db::{AstDatabase, HirDatabase}};
use hir_def::{ModuleId, FunctionLoc};
use rustc_hash::FxHashMap;
use stdx::always;
use syntax::ast;
use crate::helpers::visit_file_defs;

#[derive(PartialEq, Eq, Debug, Clone)]
enum RunnableFuncKind {
    Test,
    Bench,
    Bin,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Runnable {
    Module {
        location: ModuleId,
    },
    Function {
        kind: RunnableFuncKind,
        location: FunctionLoc,
    }
}

type WorkspaceRunnables = FxHashMap<Crate, CrateRunnables>;
type CrateRunnables = FxHashMap<FileId, FileRunnables>;
type FileRunnables = Vec<Runnable>;

// TODO: Dirty code, probably it should be, for example, member of [hir::Crate] 
fn crate_source_root<DB>(db: DB, krate: Crate) -> Arc<SourceRoot> 
where DB: HirDatabase + AstDatabase {
    let module = krate.root_module(db);
    let file_id = module.definition_source(db).file_id;
    let file_id = file_id.original_file(db);
    let source_root_id = db.file_source_root(file_id);
    db.source_root(source_root_id)
}

#[salsa::query_group(RunnableDatabaseStorage)]
pub trait RunnableDatabase: hir::db::HirDatabase + Upcast<dyn hir::db::HirDatabase> + SourceDatabaseExt {
    fn workspace_runnables(&self) -> WorkspaceRunnables;
    fn crate_runnables(&self, krait: Crate) -> CrateRunnables;
    fn file_runnables(&self, file_id: FileId) -> FileRunnables;
}

fn workspace_runnables(db: &dyn RunnableDatabase) -> WorkspaceRunnables {
    let mut res = WorkspaceRunnables::default();
    for krate in Crate::all(db.upcast()) {
        if !crate_source_root(db, krate).is_library {
            res[&krate] = db.crate_runnables(krate); 
        }
    }
    res
}

fn crate_runnables(db: &dyn RunnableDatabase, krate: Crate) -> CrateRunnables {
    let source_root = crate_source_root(db, krate);
    
    let mut res = CrateRunnables::default();
    for file_id in source_root.iter() {
        res[&file_id] = db.file_runnables(file_id);
    }
    res
}

fn file_runnables(db: &dyn RunnableDatabase, file_id: FileId) -> FileRunnables {
    let sema = Semantics::new(db);

    let mut res = Vec::new();
    // Record all runnables that come from macro expansions here instead.
    // In case an expansion creates multiple runnables we want to name them to avoid emitting a bunch of equally named runnables.
    let mut in_macro_expansion = FxHashMap::<hir::HirFileId, Vec<Runnable>>::default();
    let mut add_opt = |runnable: Option<Runnable>, def| {
        if let Some(runnable) = runnable.filter(|runnable| {
            always!(
                runnable.nav.file_id == file_id,
                "tried adding a runnable pointing to a different file: {:?} for {:?}",
                runnable.kind,
                file_id
            )
        }) {
            if let Some(def) = def {
                let file_id = match def {
                    hir::ModuleDef::Module(it) => it.declaration_source(db.upcast()).map(|src| src.file_id),
                    hir::ModuleDef::Function(it) => it.source(db.upcast()).map(|src| src.file_id),
                    _ => None,
                };
                if let Some(file_id) = file_id.filter(|file| file.call_node(db).is_some()) {
                    in_macro_expansion.entry(file_id).or_default().push(runnable);
                    return;
                }
            }
            res.push(runnable);
        }
    };
    visit_file_defs(&sema, file_id, &mut |def| match def {
        Either::Left(def) => {
            let runnable = match def {
                hir::ModuleDef::Module(it) => runnable_mod(&sema, it),
                hir::ModuleDef::Function(it) => runnable_fn(&sema, it),
                _ => None,
            };
            add_opt(runnable.or_else(|| module_def_doctest(sema.db, def)), Some(def));
        }
        Either::Right(impl_) => {
            add_opt(runnable_impl(&sema, &impl_), None);
            impl_
                .items(db.upcast())
                .into_iter()
                .map(|assoc| {
                    (
                        match assoc {
                            hir::AssocItem::Function(it) => runnable_fn(&sema, it)
                                .or_else(|| module_def_doctest(sema.db, it.into())),
                            hir::AssocItem::Const(it) => module_def_doctest(sema.db, it.into()),
                            hir::AssocItem::TypeAlias(it) => module_def_doctest(sema.db, it.into()),
                        },
                        assoc,
                    )
                })
                .for_each(|(r, assoc)| add_opt(r, Some(assoc.into())));
        }
    });

    sema.to_module_defs(file_id)
        .map(|it| runnable_mod_outline_definition(&sema, it))
        .for_each(|it| add_opt(it, None));

    res.extend(in_macro_expansion.into_iter().flat_map(|(_, runnables)| {
        let use_name_in_title = runnables.len() != 1;
        runnables.into_iter().map(move |mut r| {
            r.use_name_in_title = use_name_in_title;
            r
        })
    }));
    res
}

fn runnable_impl(sema: &Semantics<RootDatabase>, def: &hir::Impl) -> Option<Runnable> {
    let attrs = def.attrs(sema.db);
    if !has_runnable_doc_test(&attrs) {
        return None;
    }
    let cfg = attrs.cfg();
    let nav = def.try_to_nav(sema.db)?;
    let ty = def.self_ty(sema.db);
    let adt_name = ty.as_adt()?.name(sema.db);
    let mut ty_args = ty.type_arguments().peekable();
    let params = if ty_args.peek().is_some() {
        format!("<{}>", ty_args.format_with(", ", |ty, cb| cb(&ty.display(sema.db))))
    } else {
        String::new()
    };
    let test_id = TestId::Path(format!("{}{}", adt_name, params));

    Some(Runnable { use_name_in_title: false, nav, kind: RunnableKind::DocTest { test_id }, cfg })
}

fn runnable_mod(sema: &Semantics<RootDatabase>, def: hir::Module) -> Option<Runnable> {
    if !has_test_function_or_multiple_test_submodules(sema, &def) {
        return None;
    }

    // TODO: 
    // let path =
    //     def.path_to_root(sema.db).into_iter().rev().filter_map(|it| it.name(sema.db)).join("::");

    let attrs = def.attrs(sema.db);
    let cfg = attrs.cfg();
    // TODO: 
    // let nav = NavigationTarget::from_module_to_decl(sema.db, def);
    Some(Runnable { use_name_in_title: false, nav, kind: RunnableKind::TestMod { path }, cfg })
}

/// Checks if a [hir::Function] is runnable and if it is, then construct [Runnable] from it 
fn runnable_fn(sema: &Semantics<RootDatabase>, def: hir::Function) -> Option<Runnable> {
    let func = def.source(sema.db)?;
    let name_string = def.name(sema.db).to_string();

    let root = def.module(sema.db).krate().root_module(sema.db);

    let kind = if name_string == "main" && def.module(sema.db) == root {
        RunnableKind::Bin
    } else {
        let canonical_path = {
            let def: hir::ModuleDef = def.into();
            def.canonical_path(sema.db)
        };
        let test_id = canonical_path.map(TestId::Path).unwrap_or(TestId::Name(name_string));

        if test_related_attribute(&func.value).is_some() {
            let attr = TestAttr::from_fn(&func.value);
            RunnableKind::Test { test_id, attr }
        } else if func.value.has_atom_attr("bench") {
            RunnableKind::Bench { test_id }
        } else {
            return None;
        }
    };

    let nav = NavigationTarget::from_named(
        sema.db,
        func.as_ref().map(|it| it as &dyn ast::NameOwner),
        SymbolKind::Function,
    );
    let cfg = def.attrs(sema.db).cfg();
    Some(Runnable { use_name_in_title: false, nav, kind, cfg })
}

/// This is a method with a heuristics to support test methods annotated with custom test annotations, such as
/// `#[test_case(...)]`, `#[tokio::test]` and similar.
/// Also a regular `#[test]` annotation is supported.
///
/// It may produce false positives, for example, `#[wasm_bindgen_test]` requires a different command to run the test,
/// but it's better than not to have the runnables for the tests at all.
pub fn test_related_attribute(fn_def: &ast::Fn) -> Option<ast::Attr> {
    fn_def.attrs().find_map(|attr| {
        let path = attr.path()?;
        path.syntax().text().to_string().contains("test").then(|| attr)
    })
}

// We could create runnables for modules with number_of_test_submodules > 0,
// but that bloats the runnables for no real benefit, since all tests can be run by the submodule already
fn has_test_function_or_multiple_test_submodules(
    sema: &Semantics<RootDatabase>,
    module: &hir::Module,
) -> bool {
    let mut number_of_test_submodules = 0;

    for item in module.declarations(sema.db) {
        match item {
            hir::ModuleDef::Function(f) => {
                if let Some(it) = f.source(sema.db) {
                    if test_related_attribute(&it.value).is_some() {
                        return true;
                    }
                }
            }
            hir::ModuleDef::Module(submodule) => {
                if has_test_function_or_multiple_test_submodules(sema, &submodule) {
                    number_of_test_submodules += 1;
                }
            }
            _ => (),
        }
    }

    number_of_test_submodules > 1
}

const RUSTDOC_FENCE: &str = "```";
const RUSTDOC_CODE_BLOCK_ATTRIBUTES_RUNNABLE: &[&str] =
    &["", "rust", "should_panic", "edition2015", "edition2018", "edition2021"];

/// Checks that the attributes contain documentation that contain 
/// specially formed code blocks 
fn has_runnable_doc_test(attrs: &hir::Attrs) -> bool {
    attrs.docs().map_or(false, |doc| {
        for line in String::from(doc).lines() {
            if let Some(header) = line.strip_prefix(RUSTDOC_FENCE) {
                if header
                        .split(',')
                        .all(|sub| RUSTDOC_CODE_BLOCK_ATTRIBUTES_RUNNABLE.contains(&sub.trim()))
                {
                    return true;
                }
            }
        }

        false
    })
}