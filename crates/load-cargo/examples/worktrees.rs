//! Loads a checkout, then adds copies of it as worktrees, and reports what each of them costs.
//!
//! ```text
//! cargo run --release -p load-cargo --example worktrees -- <base checkout> <worktree>...
//! ```
//!
//! With `--plain`, the worktrees are loaded as ordinary workspaces that share nothing but
//! libraries, to compare with.

#![allow(clippy::print_stdout)]

use std::time::Instant;

use ide_db::{FxHashMap, RootDatabase, base_db::all_crates, prime_caches::parallel_prime_caches};
use load_cargo::{LoadCargoConfig, ProcMacroServerChoice, worktree::Overlay, worktrees::Worktrees};
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use vfs::AbsPathBuf;

fn main() -> anyhow::Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let plain = args.iter().any(|arg| arg == "--plain");
    args.retain(|arg| arg != "--plain");
    let [base, worktree_roots @ ..] = args.as_slice() else {
        anyhow::bail!("usage: worktrees [--plain] <base checkout> <worktree>...");
    };
    let threads = std::thread::available_parallelism().map_or(1, usize::from).min(16);
    let cargo_config =
        CargoConfig { sysroot: Some(RustLibSource::Discover), ..CargoConfig::default() };
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: threads,
        proc_macro_processes: 1,
    };
    let abs = |path: &str| AbsPathBuf::assert_utf8(std::fs::canonicalize(path).unwrap());
    let load = |root: &AbsPathBuf| -> anyhow::Result<ProjectWorkspace> {
        let manifest = ProjectManifest::discover_single(root)?;
        ProjectWorkspace::load(manifest, &cargo_config, &|_| {})
    };
    // Name resolution of every crate and the symbol index, which is what a client waits for.
    let index = |db: &RootDatabase| parallel_prime_caches(db, &all_crates(db), threads, &|_| ());

    let base_root = abs(base);
    let start = Instant::now();
    let (mut worktrees, mut db, mut vfs) =
        Worktrees::load(load(&base_root)?, &FxHashMap::default(), &load_config)?;
    index(&db);
    let mut crates = all_crates(&db).len();
    let mut rss = rss_mb();
    println!(
        "{:40} {:6.1}s  {crates:5} crates  {rss:6} MB",
        "base checkout",
        start.elapsed().as_secs_f64()
    );

    for worktree_root in worktree_roots {
        let worktree_root = abs(worktree_root);
        let start = Instant::now();
        let overlay = Overlay {
            worktree_root: worktree_root.clone(),
            // Nothing is under the base checkout of a plain copy, so nothing is compared.
            base_root: if plain { worktree_root.clone() } else { base_root.clone() },
        };
        // With the same manifests as the base checkout, the workspace need not be loaded.
        let workspace = match worktrees.workspace_of_copy(&overlay).filter(|_| !plain) {
            Some(workspace) => workspace,
            None => load(&worktree_root)?,
        };
        worktrees.add(&mut db, &mut vfs, workspace, overlay);
        index(&db);
        let (new_crates, new_rss) = (all_crates(&db).len(), rss_mb());
        let files = vfs
            .iter()
            .filter(|(_, path)| path.as_path().is_some_and(|it| it.starts_with(&worktree_root)))
            .count();
        println!(
            "{:40} {:6.1}s  {:+5} crates  {:+6} MB  {files:5} files loaded",
            format!("+ {}", worktree_root.file_name().unwrap_or_default()),
            start.elapsed().as_secs_f64(),
            new_crates as i64 - crates as i64,
            new_rss - rss,
        );
        (crates, rss) = (new_crates, new_rss);
    }
    println!("{:40} {:7}  {crates:5} crates  {rss:6} MB", "total", "");
    Ok(())
}

/// Resident memory of this process, where the system tells.
fn rss_mb() -> i64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
            line.split_whitespace().nth(1)?.parse::<i64>().ok()
        })
        .map_or(0, |kb| kb / 1024)
}
