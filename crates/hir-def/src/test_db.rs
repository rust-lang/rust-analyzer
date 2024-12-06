//! Database used for testing `hir_def`.

use std::{fmt, hash::BuildHasherDefault, panic, sync::Mutex};

use base_db::{
    AnchoredPath, CrateId, FileLoader, FileText, RootQueryDb, SourceDatabase, SourceRoot,
    SourceRootId, SourceRootInput, Upcast,
};
use dashmap::DashMap;
use hir_expand::{db::ExpandDatabase, files::FilePosition, InFile};
use rustc_hash::FxHasher;
use salsa::Durability;
use span::{EditionedFileId, FileId};
use syntax::{algo, ast, AstNode};
use triomphe::Arc;

use crate::{
    db::DefDatabase,
    nameres::{DefMap, ModuleSource},
    src::HasSource,
    LocalModuleId, Lookup, ModuleDefId, ModuleId,
};

#[salsa::db]
#[derive(Clone)]
pub(crate) struct TestDB {
    storage: salsa::Storage<Self>,
    files: DashMap<vfs::FileId, FileText, BuildHasherDefault<FxHasher>>,
    source_roots: DashMap<vfs::FileId, SourceRootInput, BuildHasherDefault<FxHasher>>,
    events: Arc<Mutex<Option<Vec<salsa::Event>>>>,
}

impl Default for TestDB {
    fn default() -> Self {
        let mut this = Self {
            storage: Default::default(),
            events: Default::default(),
            files: Default::default(),
            source_roots: Default::default(),
        };
        hir_expand::db::setup_syntax_context_root(this.upcast());
        this.set_expand_proc_attr_macros_with_durability(true, Durability::HIGH);
        this
    }
}

impl Upcast<dyn ExpandDatabase> for TestDB {
    #[inline]
    fn upcast(&self) -> &(dyn ExpandDatabase + 'static) {
        self
    }
}

impl Upcast<dyn DefDatabase> for TestDB {
    #[inline]
    fn upcast(&self) -> &(dyn DefDatabase + 'static) {
        self
    }
}

impl Upcast<dyn RootQueryDb> for TestDB {
    fn upcast(&self) -> &(dyn RootQueryDb + 'static) {
        self
    }
}

impl Upcast<dyn SourceDatabase> for TestDB {
    fn upcast(&self) -> &(dyn SourceDatabase + 'static) {
        self
    }
}

#[salsa::db]
impl salsa::Database for TestDB {
    fn salsa_event(&self, event: &dyn std::ops::Fn() -> salsa::Event) {
        let mut events = self.events.lock().unwrap();
        if let Some(events) = &mut *events {
            events.push(event());
        }
    }
}

impl fmt::Debug for TestDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestDB").finish()
    }
}

impl panic::RefUnwindSafe for TestDB {}

impl FileLoader for TestDB {
    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId> {
        // FileLoaderDelegate(self).resolve_path(path)
        todo!()
    }
    fn relevant_crates(&self, file_id: FileId) -> Arc<[CrateId]> {
        // FileLoaderDelegate(self).relevant_crates(file_id)
        todo!()
    }
}

#[salsa::db]
impl SourceDatabase for TestDB {
    fn file_text(&self, file_id: vfs::FileId) -> FileText {
        *self.files.get(&file_id).expect("Unable to fetch file; this is a bug")
    }

    fn set_file_text(&self, file_id: vfs::FileId, text: &str) {
        self.files.insert(file_id, FileText::new(self, file_id, Arc::from(text)));
    }

    fn set_file_text_with_durability(
        &self,
        file_id: vfs::FileId,
        text: &str,
        durability: Durability,
    ) {
        self.files.insert(
            file_id,
            FileText::builder(file_id, Arc::from(text)).durability(durability).new(self),
        );
    }

    /// Source root of the file.
    fn source_root(&self, file_id: vfs::FileId) -> SourceRootInput {
        let source_root =
            self.source_roots.get(&file_id).expect("Unable to fetch source root id; this is a bug");

        *source_root
    }

    fn set_source_root_with_durability(
        &self,
        file_id: vfs::FileId,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    ) {
        let input =
            SourceRootInput::builder(source_root_id, source_root).durability(durability).new(self);
        self.source_roots.insert(file_id, input);
    }
}

impl TestDB {
    pub(crate) fn fetch_test_crate(&self) -> CrateId {
        let crate_graph = self.crate_graph();
        let it = crate_graph
            .iter()
            .find(|&idx| {
                crate_graph[idx].display_name.as_ref().map(|it| it.canonical_name().as_str())
                    == Some("ra_test_fixture")
            })
            .or_else(|| crate_graph.iter().next())
            .unwrap();
        it
    }

    pub(crate) fn module_for_file(&self, file_id: FileId) -> ModuleId {
        let db = <TestDB as Upcast<dyn DefDatabase>>::upcast(self);

        for &krate in self.relevant_crates(file_id).iter() {
            let crate_def_map = db.crate_def_map(krate);
            for (local_id, data) in crate_def_map.modules() {
                if data.origin.file_id().map(EditionedFileId::file_id) == Some(file_id) {
                    return crate_def_map.module_id(local_id);
                }
            }
        }
        panic!("Can't find module for file")
    }

    pub(crate) fn module_at_position(&self, position: FilePosition) -> ModuleId {
        let db = <TestDB as Upcast<dyn DefDatabase>>::upcast(self);

        let file_module = self.module_for_file(position.file_id.file_id());
        let mut def_map = file_module.def_map(db);
        let module = self.mod_at_position(&def_map, position);

        def_map = match self.block_at_position(&def_map, position) {
            Some(it) => it,
            None => return def_map.module_id(module),
        };
        loop {
            let new_map = self.block_at_position(&def_map, position);
            match new_map {
                Some(new_block) if !Arc::ptr_eq(&new_block, &def_map) => {
                    def_map = new_block;
                }
                _ => {
                    // FIXME: handle `mod` inside block expression
                    return def_map.module_id(DefMap::ROOT);
                }
            }
        }
    }

    /// Finds the smallest/innermost module in `def_map` containing `position`.
    fn mod_at_position(&self, def_map: &DefMap, position: FilePosition) -> LocalModuleId {
        let db = <TestDB as Upcast<dyn DefDatabase>>::upcast(self);
        let mut size = None;
        let mut res = DefMap::ROOT;
        for (module, data) in def_map.modules() {
            let src = data.definition_source(db);
            if src.file_id != position.file_id {
                continue;
            }

            let range = match src.value {
                ModuleSource::SourceFile(it) => it.syntax().text_range(),
                ModuleSource::Module(it) => it.syntax().text_range(),
                ModuleSource::BlockExpr(it) => it.syntax().text_range(),
            };

            if !range.contains(position.offset) {
                continue;
            }

            let new_size = match size {
                None => range.len(),
                Some(size) => {
                    if range.len() < size {
                        range.len()
                    } else {
                        size
                    }
                }
            };

            if size != Some(new_size) {
                size = Some(new_size);
                res = module;
            }
        }

        res
    }

    fn block_at_position(&self, def_map: &DefMap, position: FilePosition) -> Option<Arc<DefMap>> {
        let db = <TestDB as Upcast<dyn DefDatabase>>::upcast(self);
        // Find the smallest (innermost) function in `def_map` containing the cursor.
        let mut size = None;
        let mut fn_def = None;
        for (_, module) in def_map.modules() {
            let file_id = module.definition_source(db).file_id;
            if file_id != position.file_id {
                continue;
            }
            for decl in module.scope.declarations() {
                if let ModuleDefId::FunctionId(it) = decl {
                    let range = it.lookup(db).source(db).value.syntax().text_range();

                    if !range.contains(position.offset) {
                        continue;
                    }

                    let new_size = match size {
                        None => range.len(),
                        Some(size) => {
                            if range.len() < size {
                                range.len()
                            } else {
                                size
                            }
                        }
                    };
                    if size != Some(new_size) {
                        size = Some(new_size);
                        fn_def = Some(it);
                    }
                }
            }
        }

        // Find the innermost block expression that has a `DefMap`.
        let def_with_body = fn_def?.into();
        let source_map = db.body_with_source_map(def_with_body).1;
        let scopes = db.expr_scopes(def_with_body);
        let root = db.parse(position.file_id);

        let scope_iter = algo::ancestors_at_offset(&root.syntax_node(), position.offset)
            .filter_map(|node| {
                let block = ast::BlockExpr::cast(node)?;
                let expr = ast::Expr::from(block);
                let expr_id = source_map
                    .node_expr(InFile::new(position.file_id.into(), &expr))?
                    .as_expr()
                    .unwrap();
                let scope = scopes.scope_for(expr_id).unwrap();
                Some(scope)
            });

        for scope in scope_iter {
            let mut containing_blocks =
                scopes.scope_chain(Some(scope)).filter_map(|scope| scopes.block(scope));

            if let Some(block) = containing_blocks.next().map(|block| db.block_def_map(block)) {
                return Some(block);
            }
        }

        None
    }

    pub(crate) fn log(&self, f: impl FnOnce()) -> Vec<salsa::Event> {
        *self.events.lock().unwrap() = Some(Vec::new());
        f();
        self.events.lock().unwrap().take().unwrap()
    }

    pub(crate) fn log_executed(&self, f: impl FnOnce()) -> Vec<String> {
        let events = self.log(f);
        events
            .into_iter()
            .filter_map(|e| match e.kind {
                // This is pretty horrible, but `Debug` is the only way to inspect
                // QueryDescriptor at the moment.
                salsa::EventKind::WillExecute { database_key } => {
                    Some(format!("{:?}", database_key.key_index()))
                }
                _ => None,
            })
            .collect()
    }
}
