//! rust-analyzer is lazy and doesn't compute anything unless asked. This
//! sometimes is counter productive when, for example, the first goto definition
//! request takes longer to compute. This modules implemented prepopulation of
//! various caches, it's not really advanced at the moment.

use hir::db::DefDatabase;
use ide_db::base_db::{CrateId, SourceDatabase};

use crate::RootDatabase;

/// We started indexing a crate.
#[derive(Debug)]
pub struct PrimeCachesProgress {
    pub on_crate: String,
    pub n_done: usize,
    pub n_total: usize,
}

pub(crate) fn prime_caches(db: &RootDatabase, cb: &(dyn Fn(PrimeCachesProgress) + Sync)) {
    let _p = profile::span("prime_caches");
    let graph = db.crate_graph();
    let topo = &graph.crates_in_topological_order();

    // FIXME: This would be easy to parallelize, since it's in the ideal ordering for that.
    // Unfortunately rayon prevents panics from propagation out of a `scope`, which breaks
    // cancellation, so we cannot use rayon.
    for (i, &crate_id) in topo.iter().enumerate() {
        let crate_name = graph[crate_id].display_name.as_deref().unwrap_or_default().to_string();

        cb(PrimeCachesProgress { on_crate: crate_name, n_done: i, n_total: topo.len() });
        let t1 = {
            let s = std::time::Instant::now();
            db.crate_def_map(crate_id);
            s.elapsed()
        };

        let t2 = {
            let s = std::time::Instant::now();
            db.import_map(crate_id);
            s.elapsed()
        };

        eprintln!("{:.2?} {:.2?}", t1, t2)
    }
}

pub struct PrimeCachesWorkChunk {
    krate: CrateId,
}

pub(crate) fn prime_caches_prepare_work(db: &RootDatabase) -> Vec<PrimeCachesWorkChunk> {
    let _p = profile::span("prime_caches_prepare");
    let graph = db.crate_graph();
    graph
        .crates_in_topological_order()
        .into_iter()
        .map(|krate| PrimeCachesWorkChunk { krate })
        .collect()
}

pub(crate) fn prime_caches_do_work(db: &RootDatabase, work: PrimeCachesWorkChunk) {
    let _p = profile::span("prepare_prime_caches_do_work");
    let crate_id = work.krate;
    db.crate_def_map(crate_id);
    db.import_map(crate_id);
}
