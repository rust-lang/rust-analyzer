use base_db::{FileId, Upcast, salsa};
use either::Either;
use hir::{Crate, Semantics};
use hir_def::{ModuleId, FunctionLoc};
use rustc_hash::FxHashMap;
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

#[salsa::query_group(RunnableDatabaseStorage)]
pub trait RunnableDatabase: hir::db::HirDatabase + Upcast<dyn hir::db::HirDatabase> {
    fn workspace_runnables(&self) -> WorkspaceRunnables;
    fn crate_runnables(&self, krait: Crate) -> CrateRunnables;
    fn file_runnables(&self, file_id: FileId) -> FileRunnables;
}

fn workspace_runnables(db: &dyn RunnableDatabase) -> WorkspaceRunnables {
    let mut res = WorkspaceRunnables::default();
    for krate in Crate::all(db.upcast()) {
        res[&krate] = db.crate_runnables(krate); 
    }
    res
}

fn crate_runnables(db: &dyn RunnableDatabase, krate: Crate) -> CrateRunnables {
    let krate = Crate::from(krate);
    let module = krate.root_module(db.upcast());
    let file_id = module.definition_source(db.upcast()).file_id;
    let file_id = file_id.original_file(db.upcast());
    let source_root_id = db.file_source_root(file_id);
    let source_root = db.source_root(source_root_id);
    
    let mut res = CrateRunnables::default();
    for file_id in source_root.iter() {
        res[file_id] = db.file_runnables(file_id);
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
                    hir::ModuleDef::Module(it) => it.declaration_source(db).map(|src| src.file_id),
                    hir::ModuleDef::Function(it) => it.source(db).map(|src| src.file_id),
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
                .items(db)
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

const RUSTDOC_FENCE: &str = "```";
const RUSTDOC_CODE_BLOCK_ATTRIBUTES_RUNNABLE: &[&str] =
    &["", "rust", "should_panic", "edition2015", "edition2018", "edition2021"];

/// Checks that the attributes contain specially formed documentation
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