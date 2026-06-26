//! Defines a unit of change that can applied to the database to get the next
//! state. Changes are transactional.

use std::fmt;

use rustc_hash::FxHashSet;
use salsa::{Durability, Setter as _};
use triomphe::Arc;
use vfs::FileId;

use crate::{
    CrateGraphBuilder, CratesIdMap, LibraryRoots, LocalRoots, SourceDatabase, SourceRoot,
    SourceRootId, SourceRootKind,
};

/// Encapsulate a bunch of raw `.set` calls on the database.
#[derive(Default)]
pub struct FileChange {
    pub roots: Option<Vec<(SourceRootId, SourceRoot)>>,
    pub files_changed: Vec<(FileId, Option<String>, SourceRootKind)>,
    pub crate_graph: Option<CrateGraphBuilder>,
}

impl fmt::Debug for FileChange {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = fmt.debug_struct("Change");
        if let Some(roots) = &self.roots {
            d.field("roots", roots);
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
    pub fn set_roots(&mut self, roots: Vec<(SourceRootId, SourceRoot)>) {
        self.roots = Some(roots);
    }

    pub fn change_file(&mut self, file_id: FileId, new_text: Option<String>, kind: SourceRootKind) {
        self.files_changed.push((file_id, new_text, kind))
    }

    pub fn set_crate_graph(&mut self, graph: CrateGraphBuilder) {
        self.crate_graph = Some(graph);
    }

    pub fn apply(self, db: &mut dyn SourceDatabase) -> Option<CratesIdMap> {
        let _p = tracing::info_span!("FileChange::apply").entered();
        if let Some(roots) = self.roots {
            let mut local_roots = FxHashSet::default();
            let mut library_roots = FxHashSet::default();
            for (root_id, root) in roots {
                if root.is_library {
                    library_roots.insert(root_id);
                } else {
                    local_roots.insert(root_id);
                }
                // Library source roots (and their file mappings) are stored with `NEVER_CHANGE`
                // durability and are assumed immutable, so we emit them only once. Repartitioning
                // re-produces them on every structural change; re-setting them would panic.
                if root.is_library && db.is_source_root_initialized(root_id) {
                    continue;
                }
                let durability = source_root_durability(&root);
                for file_id in root.iter() {
                    db.set_file_source_root_with_durability(file_id, root_id, durability);
                }

                db.set_source_root_with_durability(root_id, Arc::new(root), durability);
            }
            LocalRoots::get(db).set_roots(db).to(local_roots);
            LibraryRoots::get(db).set_roots(db).to(library_roots);
        }

        for (file_id, text, kind) in self.files_changed {
            let durability = kind.file_text_durability();
            // XXX: can't actually remove the file, just reset the text
            let text = text.unwrap_or_default();
            db.set_file_text_with_durability(file_id, &text, durability)
        }

        if let Some(crate_graph) = self.crate_graph {
            return Some(crate_graph.set_in_db(db));
        }
        None
    }
}

fn source_root_durability(source_root: &SourceRoot) -> Durability {
    if source_root.is_library { Durability::NEVER_CHANGE } else { Durability::MEDIUM }
}
