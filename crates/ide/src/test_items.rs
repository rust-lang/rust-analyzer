// This mod is mainly to support vscode native test extension
// please reference: https://code.visualstudio.com/api/extension-guides/testing
// It's a pretty rough implementation for now, reuse a lot of logic from runnable.
use ide_db::{base_db::FileId, RootDatabase};

use crate::{runnables::runnables, Runnable, RunnableKind};

// Feature: Test-Like Runnables
//
// Return runnables which would be shown in test explorer
// And there is no entry for editor, this method should only be called by test explorer though API directly
pub(crate) fn test_runnables_in_file(db: &RootDatabase, file_id: FileId) -> Vec<Runnable> {
    // REVIEW: We could also filter in the client side, which is better?
    return test_runnables_in_file_iter(db, file_id).collect();
}

fn test_runnables_in_file_iter(
    db: &RootDatabase,
    file_id: FileId,
) -> impl Iterator<Item = Runnable> {
    let all_runnables = runnables(db, file_id);
    let tests = all_runnables.into_iter().filter(is_test_runnable);
    return tests;

    fn is_test_runnable(runnable: &Runnable) -> bool {
        match runnable.kind {
            RunnableKind::Test { .. } => true,
            RunnableKind::TestMod { .. } => true,
            _ => false,
        }
    }
}
