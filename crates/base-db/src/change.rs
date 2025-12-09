//! Defines a unit of change that can applied to the database to get the next
//! state. Changes are transactional.

use std::fmt;

use rustc_hash::FxHashMap;

use crate::{CrateGraphBuilder, CratesIdMap, File, FileRegistration, LineEndings, SourceDatabase};

/// Encapsulate a bunch of raw `.set` calls on the database.
#[derive(Default)]
pub struct FileChange {
    pub indexed_files: Option<Vec<FileRegistration>>,
    pub files_changed: Vec<(File, Option<String>)>,
    pub crate_graph: Option<CrateGraphBuilder>,
}

impl fmt::Debug for FileChange {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = fmt.debug_struct("Change");
        if let Some(indexed_files) = &self.indexed_files {
            d.field("indexed_files", indexed_files);
        }
        if !self.files_changed.is_empty() {
            d.field("files_changed", &self.files_changed.len());
        }
        if self.crate_graph.is_some() {
            d.field("crate_graph", &self.crate_graph);
        }
        d.finish()
    }
}

impl FileChange {
    pub fn set_indexed_files(&mut self, files: Vec<FileRegistration>) {
        self.indexed_files = Some(files);
    }

    pub fn change_file(&mut self, file: File, new_text: Option<String>) {
        self.files_changed.push((file, new_text))
    }

    pub fn set_crate_graph(&mut self, graph: CrateGraphBuilder) {
        self.crate_graph = Some(graph);
    }

    pub fn apply(self, db: &mut dyn SourceDatabase) -> Option<CratesIdMap> {
        let _p = tracing::info_span!("FileChange::apply").entered();
        let FileChange { indexed_files, files_changed, crate_graph } = self;
        let mut file_durability = FxHashMap::default();
        if let Some(indexed_files) = &indexed_files {
            for registration in indexed_files {
                file_durability.insert(registration.file, registration.root.kind.text_durability());
            }
        }
        for (file, _) in &files_changed {
            if file_durability.contains_key(file) {
                continue;
            }
            if let Some(root) = db.file_root(*file) {
                file_durability.insert(*file, root.kind.text_durability());
            }
        }
        if let Some(indexed_files) = indexed_files {
            db.set_indexed_files(indexed_files);
        }

        for (file, text) in files_changed {
            let durability = file_durability.get(&file).copied().unwrap_or_else(|| {
                let Some(root) = db.file_root(file) else {
                    panic!("Unable to fetch file root for {file:?}; this is a bug")
                };
                root.kind.text_durability()
            });
            let (text, line_endings) = match text {
                Some(text) => LineEndings::normalize(text),
                None => (String::new(), LineEndings::Unix),
            };
            db.set_file_text_with_line_endings_and_durability(file, &text, line_endings, durability)
        }

        if let Some(crate_graph) = crate_graph {
            return Some(crate_graph.set_in_db(db));
        }
        None
    }
}
