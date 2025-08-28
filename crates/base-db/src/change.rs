//! Defines a unit of change that can be applied to the database to get the next
//! state. Changes are transactional.

use std::fmt;

use salsa::Durability;
use triomphe::Arc;
use vfs::FileId;

use crate::{CrateGraphBuilder, CratesIdMap, RootQueryDb, SourceRoot, SourceRootId};

/// Encapsulates a batch of raw `.set` calls on the database.
#[derive(Default)]
pub struct FileChange {
    pub roots: Option<Vec<SourceRoot>>,
    pub files_changed: Vec<(FileId, Option<String>)>,
    pub crate_graph: Option<CrateGraphBuilder>,
}

impl fmt::Debug for FileChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("FileChange");

        if let Some(roots) = &self.roots {
            d.field("roots", roots);
        }
        if !self.files_changed.is_empty() {
            d.field("files_changed_count", &self.files_changed.len());
        }
        if let Some(graph) = &self.crate_graph {
            d.field("crate_graph", graph);
        }

        d.finish()
    }
}

impl FileChange {
    // --- New clearer names ---

    /// Replace all source roots with a new set.
    #[inline]
    pub fn replace_roots(&mut self, roots: Vec<SourceRoot>) {
        self.roots = Some(roots);
    }

    /// Update the text of a file, or reset it if `None`.
    #[inline]
    pub fn update_file_text(&mut self, file_id: FileId, new_text: Option<String>) {
        self.files_changed.push((file_id, new_text));
    }

    /// Replace the crate graph.
    #[inline]
    pub fn replace_crate_graph(&mut self, graph: CrateGraphBuilder) {
        self.crate_graph = Some(graph);
    }

    /// Apply this change to the given database.
    pub fn apply(self, db: &mut dyn RootQueryDb) -> Option<CratesIdMap> {
        let _p = tracing::info_span!("FileChange::apply").entered();

        if let Some(roots) = self.roots {
            for (idx, root) in roots.into_iter().enumerate() {
                let root_id = SourceRootId(idx as u32);
                let durability = durability_for_root(root.is_library);

                for file_id in root.iter() {
                    db.set_file_source_root_with_durability(file_id, root_id, durability);
                }

                db.set_source_root_with_durability(root_id, Arc::new(root), durability);
            }
        }

        for (file_id, text) in self.files_changed {
            let source_root_id = db.file_source_root(file_id);
            let source_root = db.source_root(source_root_id.source_root_id(db));
            let durability = durability_for_file(source_root.source_root(db).is_library);

            // XXX: can't actually remove the file, just reset the text
            let text = text.unwrap_or_else(String::new);
            db.set_file_text_with_durability(file_id, &text, durability);
        }

        self.crate_graph.map(|graph| graph.set_in_db(db))
    }

    // --- Backwards-compatible aliases (keep existing call sites working) ---

    /// Back-compat: same as `replace_roots`.
    #[inline]
    pub fn set_roots(&mut self, roots: Vec<SourceRoot>) {
        self.replace_roots(roots);
    }

    /// Back-compat: same as `update_file_text`.
    #[inline]
    pub fn change_file(&mut self, file_id: FileId, new_text: Option<String>) {
        self.update_file_text(file_id, new_text);
    }

    /// Back-compat: same as `replace_crate_graph`.
    #[inline]
    pub fn set_crate_graph(&mut self, graph: CrateGraphBuilder) {
        self.replace_crate_graph(graph);
    }
}

fn durability_for_root(is_library: bool) -> Durability {
    if is_library { Durability::MEDIUM } else { Durability::LOW }
}

fn durability_for_file(is_library: bool) -> Durability {
    if is_library { Durability::HIGH } else { Durability::LOW }
}
