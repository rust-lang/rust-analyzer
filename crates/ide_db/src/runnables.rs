use std::sync::Arc;

use base_db::{FileId, SourceDatabaseExt, SourceRoot, Upcast, salsa};
use either::Either;
use hir::{Crate, HasAttrs, HasSource, Module, Semantics, db::{AstDatabase, HirDatabase}};
use hir_def::{FunctionLoc};
use rustc_hash::FxHashMap;
use stdx::{always, format_to};
use syntax::{TextRange, ast};
use crate::helpers::visit_file_defs;
use std::collections::LinkedList;

/// Defines the kind of [RunnableFunc]
#[derive(PartialEq, Eq, Debug, Clone)]
enum RunnableFuncKind {
    /// The [unit test function](https://doc.rust-lang.org/reference/attributes/testing.html?highlight=test#testing-attributes),
    /// i.e. function marked with `#[test]` attribute and whose signature satisfies requirements.
    Test,
    /// The [benchmark test function](https://doc.rust-lang.org/unstable-book/library-features/test.html),
    /// i.e. function marked with `#[bench]` attribute and whose signature satisfies requirements.
    /// Requires the unstable feature `test` to be enabled.
    Bench,
    /// It is the entry point of the crate. Default is a function with the name `main` 
    /// that signature satisfies requirements. If unstable feature 
    /// [`start`](https://doc.rust-lang.org/unstable-book/language-features/start.html?highlight=start#start)
    /// enabled, insted use function market with attribute `#[start]` that signature satisfies requirements. 
    Bin,
}

struct DoctestLocation {
    file_id: FileId,
    range: TextRange,
}

/// [Documentation tests](https://doc.rust-lang.org/rustdoc/documentation-tests.html)
/// these are special inserts into mardown that contain Rust code and can be executed 
/// as tests.
struct Doctest {
    location: DoctestLocation,
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct RunnableFunc {
    kind: RunnableFuncKind,
    location: FunctionLoc,
}

/// We can think about it as a representation a partial view from AST. 
/// The leaves of which are runnables: [RunnableFunc] and [Doctest], and 
/// the edges are Modules.
/// The main purpose of this partial view is that store runnables with 
/// respect to the project structure.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum RunnableView {
    Module {
        location: Module,
        content: LinkedList<RunnableView>,
    },
    Function(RunnableFunc),
    Doctest(Doctest),
}

type WorkspaceRunnables = FxHashMap<Crate, CrateRunnables>;
type CrateRunnables = FxHashMap<FileId, FileRunnables>;
type FileRunnables = Vec<RunnableView>;

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
    let mut in_macro_expansion = FxHashMap::<hir::HirFileId, Vec<RunnableView>>::default();
    let mut add_opt = |runnable: Option<RunnableView>, def| {
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

/// Creates a test mod runnable for outline modules at the top of their definition.
fn runnable_mod_outline_definition(
    sema: &Semantics,
    def: hir::Module,
) -> Option<RunnableView> {
    if !is_contains_runnable(sema, &def) {
        return None;
    }
    //TODO: let path =
    //TODO:    def.path_to_root(sema.db).into_iter().rev().filter_map(|it| it.name(sema.db)).join("::");

    //TODO: let attrs = def.attrs(sema.db);
    //TODO: let cfg = attrs.cfg();
    // match def.definition_source(sema.db).value {
    //     hir::ModuleSource::SourceFile(_) => Some(Runnable {
    //         use_name_in_title: false,
    //         nav: def.to_nav(sema.db),
    //         kind: RunnableKind::TestMod { path },
    //         cfg,
    //     }),
    //     _ => None,
    // }

    Some(RunnableView::Module{ location: def, content: ()})
}

/// Checks if module containe runnable in doc than create [Runnable] from it
fn module_def_doctest(db: &dyn RunnableDatabase, def: hir::ModuleDef) -> Option<RunnableView> {
    let attrs = match def {
        hir::ModuleDef::Module(it) => it.attrs(db),
        hir::ModuleDef::Function(it) => it.attrs(db),
        hir::ModuleDef::Adt(it) => it.attrs(db),
        hir::ModuleDef::Variant(it) => it.attrs(db),
        hir::ModuleDef::Const(it) => it.attrs(db),
        hir::ModuleDef::Static(it) => it.attrs(db),
        hir::ModuleDef::Trait(it) => it.attrs(db),
        hir::ModuleDef::TypeAlias(it) => it.attrs(db),
        hir::ModuleDef::BuiltinType(_) => return None,
    };
    if !is_contains_runnable_in_doc(&attrs) {
        return None;
    }
    let def_name = def.name(db)?;
    let path = (|| {
        let mut path = String::new();
        def.canonical_module_path(db)?
            .flat_map(|it| it.name(db))
            .for_each(|name| format_to!(path, "{}::", name));
        // This probably belongs to canonical_path?
        if let Some(assoc_item) = def.as_assoc_item(db) {
            if let hir::AssocItemContainer::Impl(imp) = assoc_item.container(db) {
                let ty = imp.self_ty(db);
                if let Some(adt) = ty.as_adt() {
                    let name = adt.name(db);
                    let mut ty_args = ty.type_arguments().peekable();
                    format_to!(path, "{}", name);
                    if ty_args.peek().is_some() {
                        format_to!(
                            path,
                            "<{}>",
                            ty_args.format_with(", ", |ty, cb| cb(&ty.display(db)))
                        );
                    }
                    format_to!(path, "::{}", def_name);
                    return Some(path);
                }
            }
        }
        format_to!(path, "{}", def_name);
        Some(path)
    })();

    let test_id = path.map_or_else(|| TestId::Name(def_name.to_string()), TestId::Path);

    let mut nav = match def {
        hir::ModuleDef::Module(def) => NavigationTarget::from_module_to_decl(db, def),
        def => def.try_to_nav(db)?,
    };
    nav.focus_range = None;
    nav.description = None;
    nav.docs = None;
    nav.kind = None;
    let res = RunnableView {
        use_name_in_title: false,
        nav,
        kind: RunnableKind::DocTest { test_id },
        cfg: attrs.cfg(),
    };
    Some(res)
}

/// Checks if implementation containe runnable in doc than create [Runnable] from it
fn runnable_impl(sema: &Semantics, def: &hir::Impl) -> Option<RunnableView> {
    let attrs = def.attrs(sema.db);
    if !is_contains_runnable_in_doc(&attrs) {
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
    //TODO: let test_id = TestId::Path(format!("{}{}", adt_name, params));

    // Some(Runnable { use_name_in_title: false, nav, kind: RunnableKind::DocTest { test_id }, cfg })
    todo!()
}

/// Checks if a [hir::Module] is runnable and if it is, then construct [Runnable] from it
fn runnable_mod(sema: &Semantics, def: hir::Module) -> Option<RunnableView> {
    if !is_contains_runnable(sema, &def) {
        return None;
    }
    
    Some(RunnableView::Module{ location: def, content: todo!() })
}

/// Checks if a [hir::Function] is runnable and if it is, then construct [Runnable] from it 
fn runnable_fn(sema: &Semantics, def: hir::Function) -> Option<RunnableView> {
    let func = def.source(sema.db)?;
    let name_string = def.name(sema.db).to_string();

    let root = def.module(sema.db).krate().root_module(sema.db);

    let kind = if name_string == "main" && def.module(sema.db) == root {
        RunnableFuncKind::Bin
    } else {
        let canonical_path = {
            let def: hir::ModuleDef = def.into();
            def.canonical_path(sema.db)
        };
        //TODO: let test_id = canonical_path.map(TestId::Path).unwrap_or(TestId::Name(name_string));

        if extract_test_related_attribute(&func.value).is_some() {
            //TODO: let attr = TestAttr::from_fn(&func.value);
            //TODO: RunnableKind::Test { test_id, attr }
            RunnableFuncKind::Test
        } else if func.value.has_atom_attr("bench") {
            RunnableFuncKind::Bench
            //TODO: RunnableKind::Bench { test_id }
        } else {
            return None;
        }
    };

    Some(RunnableView::Function(RunnableFunc{ kind, location: todo!() }))
}

/// This is a method with a heuristics to support test methods annotated 
/// with custom test annotations, such as `#[test_case(...)]`, 
/// `#[tokio::test]` and similar.
/// Also a regular `#[test]` annotation is supported.
///
/// It may produce false positives, for example, `#[wasm_bindgen_test]` 
/// requires a different command to run the test, but it's better than 
/// not to have the runnables for the tests at all.
pub fn extract_test_related_attribute(fn_def: &ast::Fn) -> Option<ast::Attr> {
    fn_def.attrs().find_map(|attr| {
        attr.path()?
            .syntax()
            .text()
            .to_string()
            .contains("test")
            .then(|| attr)
    })
}

/// Checks that module contains at least one runnable function or module
fn is_contains_runnable(
    sema: &Semantics,
    module: &hir::Module,
) -> bool {
    for item in module.declarations(sema.db) {
        match item {
            hir::ModuleDef::Function(f) => {
                if let Some(it) = f.source(sema.db) {
                    if extract_test_related_attribute(&it.value).is_some() {
                        return true;
                    }
                }
            }
            hir::ModuleDef::Module(submodule) => {
                if is_contains_runnable(sema, &submodule) {
                    return true;
                }
            }
            _ => (),
        }
    }

    false
}

const RUSTDOC_FENCE: &str = "```";
const RUSTDOC_CODE_BLOCK_ATTRIBUTES_RUNNABLE: &[&str] =
    &["", "rust", "should_panic", "edition2015", "edition2018", "edition2021"];

/// Checks that the attributes contain documentation that contain 
/// specially formed code blocks 
fn is_contains_runnable_in_doc(attrs: &hir::Attrs) -> bool {
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