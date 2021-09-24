use base_db::salsa;

#[salsa::query_group(RunnableDatabaseStorage)]
pub trait RunnableDatabase: hir::db::HirDatabase {
    fn workspaces_runnables(&self);
    fn crate_runnables(&self, crait_id: CrateId);
    fn file_runnables(&self, file_id: FileId);
}

fn workspaces_runnables(db: &dyn RunnableDatabase) {
    db.crate_graph();
    db.crate_runnables(crait_id);
}

fn crate_runnables(db: &dyn RunnableDatabase, crait_id: CrateId) {
    db.crate_graph()[crait_id];
    db.file_runnables(file_id);
}

fn file_runnables(db: &dyn RunnableDatabase, file_id: FileId) -> Vec<Runnable> {
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