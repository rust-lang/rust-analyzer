//! Database used for testing `hir`.

use std::{
    fmt, panic,
    sync::{Mutex, OnceLock},
};

use base_db::{
    CrateGraphBuilder, CratesMap, FileLookup, FileRegistration, FileVisibility, Files, Nonce,
    PendingFileChangeKind, SourceDatabase, VfsPath, WorkspaceFiles, all_crates, relevant_crates,
    set_all_crates_with_durability,
};

use hir_def::{ModuleId, nameres::crate_def_map};
use hir_expand::EditionedFileId;
use rustc_hash::FxHashMap;
use salsa::Durability;
use span::File;
use syntax::TextRange;
use test_utils::extract_annotations;
use triomphe::Arc;

#[salsa::db]
pub(crate) struct TestDB {
    storage: salsa::Storage<Self>,
    files: Files,
    workspace_files: OnceLock<WorkspaceFiles>,
    crates_map: Arc<CratesMap>,
    events: Arc<Mutex<Option<Vec<salsa::Event>>>>,
    nonce: Nonce,
}

impl Default for TestDB {
    fn default() -> Self {
        let events = <Arc<Mutex<Option<Vec<salsa::Event>>>>>::default();
        let mut this = Self {
            storage: salsa::Storage::new(Some(Box::new({
                let events = events.clone();
                move |event| {
                    let mut events = events.lock().unwrap();
                    if let Some(events) = &mut *events {
                        events.push(event);
                    }
                }
            }))),
            events,
            files: Files::default(),
            workspace_files: OnceLock::new(),
            crates_map: Default::default(),
            nonce: Nonce::new(),
        };
        let workspace_files = WorkspaceFiles::new(&this, Vec::new().into(), Vec::new().into());
        assert!(this.workspace_files.set(workspace_files).is_ok());
        hir_def::set_expand_proc_attr_macros(&mut this, true);
        // This needs to be here otherwise `CrateGraphBuilder` panics.
        set_all_crates_with_durability(&mut this, std::iter::empty(), Durability::HIGH);
        CrateGraphBuilder::default().set_in_db(&mut this);
        this
    }
}

impl Clone for TestDB {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            files: self.files.clone(),
            workspace_files: self.workspace_files.clone(),
            crates_map: self.crates_map.clone(),
            events: self.events.clone(),
            nonce: self.nonce,
        }
    }
}

impl fmt::Debug for TestDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestDB").finish()
    }
}

#[salsa::db]
impl SourceDatabase for TestDB {
    fn workspace_files(&self) -> WorkspaceFiles {
        *self.workspace_files.get().expect("TestDB must initialize workspace files")
    }

    fn file_data(&self, file: File) -> base_db::FileData {
        self.files.file_data(file)
    }

    fn file_path(&self, file_id: File) -> Option<VfsPath> {
        self.files.file_path(self, file_id)
    }

    fn file_for_path(&self, path: &VfsPath) -> Option<FileLookup> {
        self.files.file_for_path(self, path)
    }

    fn intern_file_path(&self, path: VfsPath) -> File {
        self.files.intern_file_path(self, path)
    }

    fn resolve_path(&self, anchor: File, paths: &[&str]) -> Option<(File, usize)> {
        base_db::resolve_path(self, anchor, paths)
    }

    fn record_file_contents(
        &mut self,
        path: VfsPath,
        contents: Option<Vec<u8>>,
        visibility: FileVisibility,
    ) {
        self.files.clone().record_file_contents(self, path, contents, visibility);
    }

    fn take_file_changes(&mut self) -> rustc_hash::FxHashMap<File, PendingFileChangeKind> {
        self.files.take_file_changes()
    }

    fn set_indexed_files(&mut self, files: Vec<FileRegistration>) {
        let workspace_files = self.workspace_files();
        workspace_files.replace(self, &self.files.clone(), files);
    }

    fn crates_map(&self) -> Arc<CratesMap> {
        self.crates_map.clone()
    }

    fn nonce_and_revision(&self) -> (Nonce, salsa::Revision) {
        (self.nonce, salsa::plumbing::ZalsaDatabase::zalsa(self).current_revision())
    }

    fn line_column(&self, _file: File, _offset: syntax::TextSize) -> Result<(u32, u32), ()> {
        Err(())
    }
}

#[salsa::db]
impl salsa::Database for TestDB {}

impl panic::RefUnwindSafe for TestDB {}

impl TestDB {
    pub(crate) fn module_for_file_opt(&self, file_id: impl Into<File>) -> Option<ModuleId> {
        let file_id = file_id.into();
        for &krate in relevant_crates(self, file_id).iter() {
            let crate_def_map = crate_def_map(self, krate);
            for (module_id, data) in crate_def_map.modules() {
                if data.origin.file_id().map(|file_id| file_id.file(self)) == Some(file_id) {
                    return Some(module_id);
                }
            }
        }
        None
    }

    pub(crate) fn module_for_file(&self, file_id: impl Into<File>) -> ModuleId {
        self.module_for_file_opt(file_id.into()).unwrap()
    }

    pub(crate) fn extract_annotations(
        &self,
    ) -> FxHashMap<EditionedFileId, Vec<(TextRange, String)>> {
        let mut files = Vec::new();
        for &krate in all_crates(self).iter() {
            let crate_def_map = crate_def_map(self, krate);
            for (module_id, _) in crate_def_map.modules() {
                let file_id = crate_def_map[module_id].origin.file_id();
                files.extend(file_id)
            }
        }
        files
            .into_iter()
            .filter_map(|file_id| {
                let text = self.file_data(file_id.file(self));
                let annotations = extract_annotations(text.text(self));
                if annotations.is_empty() {
                    return None;
                }
                Some((file_id, annotations))
            })
            .collect()
    }
}

impl TestDB {
    pub(crate) fn log(&self, f: impl FnOnce()) -> Vec<salsa::Event> {
        *self.events.lock().unwrap() = Some(Vec::new());
        f();
        self.events.lock().unwrap().take().unwrap()
    }

    pub(crate) fn log_executed(&self, f: impl FnOnce()) -> (Vec<String>, Vec<salsa::Event>) {
        let events = self.log(f);
        let executed = events
            .iter()
            .filter_map(|e| match e.kind {
                // This is pretty horrible, but `Debug` is the only way to inspect
                // QueryDescriptor at the moment.
                salsa::EventKind::WillExecute { database_key } => {
                    let ingredient = (self as &dyn salsa::Database)
                        .ingredient_debug_name(database_key.ingredient_index());
                    Some(ingredient.to_string())
                }
                _ => None,
            })
            .collect();
        (executed, events)
    }
}
