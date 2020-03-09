//! rust-analyzer is lazy and doesn't not compute anything unless asked. This
//! sometimes is counter productive when, for example, the first goto definition
//! request takes longer to compute. This modules implemented prepopulating of
//! various caches, it's not really advanced at the moment.

use hir::db::DefDatabase;
use ra_db::{salsa::ParallelDatabase, FileLoader, SourceDatabase};
use rustc_hash::FxHashSet;

use crate::{CrateGraph, CrateId, FileId, RootDatabase};

pub(crate) fn prime_caches(db: &RootDatabase, files: Vec<FileId>) {
    let crates = files
        .into_iter()
        .flat_map(|file_id| db.relevant_crates(file_id).first().copied())
        .collect::<Vec<_>>();
    let crate_graph = db.crate_graph();
    let crates = top_sort(&crate_graph, crates);

    let _t = ra_prof::print_time("cache-priming");
    let db = db.snapshot();
    rayon::scope(move |s| {
        for krate in crates {
            let db = db.snapshot();
            s.spawn(move |_| {
                db.crate_def_map(krate);
            })
        }
    })
}

fn top_sort(graph: &CrateGraph, sources: Vec<CrateId>) -> Vec<CrateId> {
    let mut res = Vec::new();
    let mut visited = FxHashSet::default();
    for krate in sources {
        go(graph, &mut visited, &mut res, krate);
    }

    return res;

    fn go(
        graph: &CrateGraph,
        visited: &mut FxHashSet<CrateId>,
        res: &mut Vec<CrateId>,
        source: CrateId,
    ) {
        if !visited.insert(source) {
            return;
        }
        for dep in graph[source].dependencies.iter() {
            go(graph, visited, res, dep.crate_id)
        }
        res.push(source)
    }
}
