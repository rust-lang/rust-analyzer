//! Measures how analysis of a checkout and worktrees of it in one database scales with threads,
//! and what changes to the files cost the queries that run at the same time.
//!
//! ```text
//! cargo run --release -p load-cargo --example worktrees_parallel -- <mode> <base> <worktree>...
//! ```
//!
//! Modes:
//! - `prime <threads>`: time to prime the caches (name resolution and the symbol index).
//! - `cold <threads>`: time to compute the diagnostics of every file once, from nothing.
//! - `warm`: the same once everything is computed, for a growing number of threads.
//! - `scenario`: what a developer does in a worktree, step by step: edits, a change of branch,
//!   another version of a dependency. Creates the worktree `wt-scn` next to the base checkout.
//! - `isolation <copies> <changes>`: checks that what is changed in one checkout is seen there
//!   and nowhere else, through many changes to many worktrees, and reports the memory.
//! - `churn <readers> <ms between changes> <seconds> <file>`: diagnostics on `readers` threads
//!   while `file` of the first worktree, or of the base checkout without one, is changed.

#![allow(clippy::print_stdout)]

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use ide::{Analysis, AnalysisHost, AssistResolveStrategy, DiagnosticsConfig, FileId};
use ide_db::{FxHashMap, base_db::all_crates};
use load_cargo::{LoadCargoConfig, ProcMacroServerChoice, worktree::Overlay, worktrees::Worktrees};
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use vfs::{AbsPathBuf, Vfs};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, args) = args.split_first().ok_or_else(|| anyhow::format_err!("no mode"))?;
    let (options, roots): (Vec<&String>, Vec<&String>) =
        args.iter().partition(|arg| !std::path::Path::new(arg.as_str()).is_dir());
    anyhow::ensure!(!roots.is_empty(), "no checkout given");
    // Files are found by their absolute paths.
    let roots: Vec<String> = roots
        .iter()
        .map(|root| Ok(std::fs::canonicalize(root)?.to_string_lossy().into_owned()))
        .collect::<anyhow::Result<_>>()?;
    let roots: Vec<&String> = roots.iter().collect();
    let option = |idx: usize| options.get(idx).map(|it| it.as_str());
    let number =
        |idx: usize, default: usize| option(idx).and_then(|it| it.parse().ok()).unwrap_or(default);

    let start = Instant::now();
    let (mut worktrees, mut vfs, mut host, overlays) = load(&roots)?;
    // Finding the files of crates needs name resolution, which is part of what `cold` is to
    // measure: there it is done first, on the threads asked for, and counted.
    let name_resolution = Instant::now();
    if mode == "cold" {
        let crates = all_crates(host.raw_database());
        host.analysis().parallel_prime_caches(&crates, number(0, 1), |_| ()).unwrap();
    }
    let files = files_of_crates(&host, &vfs, &roots);
    let name_resolution = name_resolution.elapsed();
    println!(
        "loaded {} checkouts in {:.1}s: {} crates, {} files of local crates",
        roots.len(),
        start.elapsed().as_secs_f64(),
        all_crates(host.raw_database()).len(),
        files.len()
    );

    match mode.as_str() {
        "prime" => {
            let threads = number(0, 1);
            let start = Instant::now();
            let crates = all_crates(host.raw_database());
            host.analysis().parallel_prime_caches(&crates, threads, |_| ()).unwrap();
            println!("prime  {threads:3} threads  {:7.2}s", start.elapsed().as_secs_f64());
        }
        "cold" => {
            let threads = number(0, 1);
            let (elapsed, mut per_file) = timed_diagnostics_pass(&host, &files, threads);
            println!(
                "cold   {threads:3} threads  {:7.2}s  (name resolution {:.2}s + diagnostics {:.2}s)",
                (name_resolution + elapsed).as_secs_f64(),
                name_resolution.as_secs_f64(),
                elapsed.as_secs_f64()
            );
            // A file that takes long bounds the time whatever the number of threads; time spent
            // in files beyond what the processor was used for is time spent waiting for another
            // thread that computes the same thing.
            per_file.sort_by_key(|&(time, _)| std::cmp::Reverse(time));
            let in_files: Duration = per_file.iter().map(|&(time, _)| time).sum();
            println!("       spent in files, summed over threads: {:.1}s", in_files.as_secs_f64());
            for &(time, file) in per_file.iter().take(number(1, 0)) {
                println!("       {:6.2}s  {}", time.as_secs_f64(), vfs.file_path(file));
            }
        }
        "warm" => {
            diagnostics_pass(&host, &files, 32);
            let base = diagnostics_pass(&host, &files, 1);
            for threads in [1, 2, 4, 8, 16, 32] {
                let elapsed = diagnostics_pass(&host, &files, threads);
                println!(
                    "warm   {threads:3} threads  {:7.3}s  {:8.0} files/s  speedup {:5.2}",
                    elapsed.as_secs_f64(),
                    files.len() as f64 / elapsed.as_secs_f64(),
                    base.as_secs_f64() / elapsed.as_secs_f64(),
                );
            }
        }
        "churn" => {
            let (readers, interval, seconds) = (number(0, 8), number(1, 0), number(2, 10));
            let edited = option(3).unwrap_or("crates/stdx/src/lib.rs");
            let root = overlays.first().map_or(&roots[0][..], |it| it.worktree_root.as_str());
            let edited = AbsPathBuf::assert_utf8(std::path::Path::new(root).join(edited));
            // Everything is computed before we start, as it would be in a server that has run
            // for a while.
            diagnostics_pass(&host, &files, 32);
            churn(&mut worktrees, &mut vfs, &mut host, &files, &edited, readers, interval, seconds);
        }
        "isolation" => {
            let (copies, changes) = (number(0, 8), number(1, 200));
            drop((worktrees, vfs, host));
            isolation(roots[0], copies, changes)?;
        }
        "scenario" => {
            drop((worktrees, vfs, host));
            scenario(roots[0])?;
        }
        _ => anyhow::bail!("unknown mode {mode}"),
    }
    Ok(())
}

fn load(roots: &[&String]) -> anyhow::Result<(Worktrees, Vfs, AnalysisHost, Vec<Overlay>)> {
    let cargo_config =
        CargoConfig { sysroot: Some(RustLibSource::Discover), ..CargoConfig::default() };
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let abs = |path: &str| AbsPathBuf::assert_utf8(std::fs::canonicalize(path).unwrap());
    let load = |root: &AbsPathBuf| -> anyhow::Result<ProjectWorkspace> {
        let manifest = ProjectManifest::discover_single(root)?;
        ProjectWorkspace::load(manifest, &cargo_config, &|_| {})
    };
    let base_root = abs(roots[0]);
    let (mut worktrees, mut db, mut vfs) =
        Worktrees::load(load(&base_root)?, &FxHashMap::default(), &load_config)?;
    let mut overlays = Vec::new();
    for root in &roots[1..] {
        let overlay = Overlay { worktree_root: abs(root), base_root: base_root.clone() };
        let workspace = match worktrees.workspace_of_copy(&overlay) {
            Some(workspace) => workspace,
            None => load(&overlay.worktree_root)?,
        };
        worktrees.add(&mut db, &mut vfs, workspace, overlay.clone());
        overlays.push(overlay);
    }
    Ok((worktrees, vfs, AnalysisHost::with_database(db), overlays))
}

/// The Rust files of the checkouts that are part of a crate.
fn files_of_crates(host: &AnalysisHost, vfs: &Vfs, roots: &[&String]) -> Vec<FileId> {
    let analysis = host.analysis();
    // One file that takes very long hides how the rest scales: `SKIP` leaves out the files
    // whose path contains it.
    let skip = std::env::var("SKIP").ok();
    vfs.iter()
        .filter(|(_, path)| {
            path.as_path().is_some_and(|path| {
                path.extension() == Some("rs")
                    && !skip.as_deref().is_some_and(|skip| path.as_str().contains(skip))
                    && roots.iter().any(|root| path.as_str().starts_with(root.as_str()))
            })
        })
        .map(|(file, _)| file)
        .filter(|&file| analysis.crates_for(file).is_ok_and(|crates| !crates.is_empty()))
        .collect()
}

/// Computes the diagnostics of every one of `files` once, on `threads` threads.
fn diagnostics_pass(host: &AnalysisHost, files: &[FileId], threads: usize) -> Duration {
    timed_diagnostics_pass(host, files, threads).0
}

/// Like `diagnostics_pass`, and tells how long each file took.
fn timed_diagnostics_pass(
    host: &AnalysisHost,
    files: &[FileId],
    threads: usize,
) -> (Duration, Vec<(Duration, FileId)>) {
    let config = DiagnosticsConfig::test_sample();
    let next = AtomicUsize::new(0);
    let start = Instant::now();
    let per_file = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                let analysis = host.analysis();
                let (config, next) = (&config, &next);
                scope.spawn(move || {
                    let mut per_file = Vec::new();
                    while let Some(&file) = files.get(next.fetch_add(1, Ordering::Relaxed)) {
                        let start = Instant::now();
                        analysis
                            .full_diagnostics(config, AssistResolveStrategy::None, file)
                            .unwrap();
                        per_file.push((start.elapsed(), file));
                    }
                    per_file
                })
            })
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().unwrap()).collect()
    });
    (start.elapsed(), per_file)
}

/// Computes diagnostics on `readers` threads for `seconds`, while the file at `edited` is changed
/// every `interval` milliseconds, never if 0.
#[allow(clippy::too_many_arguments)]
fn churn(
    worktrees: &mut Worktrees,
    vfs: &mut Vfs,
    host: &mut AnalysisHost,
    files: &[FileId],
    edited: &AbsPathBuf,
    readers: usize,
    interval: usize,
    seconds: usize,
) {
    let config = DiagnosticsConfig::test_sample();
    let original = std::fs::read_to_string(edited).unwrap();
    let (done, cancelled) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let stop = std::sync::atomic::AtomicBool::new(false);
    let mut changes = Vec::new();
    let mut measured = Duration::ZERO;

    std::thread::scope(|scope| {
        // A reader works with a snapshot until a change cancels it, then waits for the next one.
        let mut snapshots = Vec::new();
        for reader in 0..readers {
            let (sender, receiver) = crossbeam_channel::unbounded::<Analysis>();
            sender.send(host.analysis()).unwrap();
            snapshots.push(sender);
            let (config, done, cancelled, stop) = (&config, &done, &cancelled, &stop);
            scope.spawn(move || {
                let mut next = reader * 7919;
                'snapshots: while let Ok(mut analysis) = receiver.recv() {
                    // Only the latest snapshot is of use: older ones are cancelled already.
                    while let Ok(newer) = receiver.try_recv() {
                        analysis = newer;
                    }
                    while !stop.load(Ordering::Relaxed) {
                        let file = files[next % files.len()];
                        next += 1;
                        match analysis.full_diagnostics(config, AssistResolveStrategy::None, file) {
                            Ok(_) => done.fetch_add(1, Ordering::Relaxed),
                            Err(_) => {
                                cancelled.fetch_add(1, Ordering::Relaxed);
                                continue 'snapshots;
                            }
                        };
                    }
                    break;
                }
            });
        }

        let start = Instant::now();
        let mut change = 0;
        while start.elapsed() < Duration::from_secs(seconds as u64) {
            if interval == 0 {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            std::thread::sleep(Duration::from_millis(interval as u64));
            change += 1;
            let text = format!("{original}\npub fn churn_{change}() {{}}\n");
            let applying = Instant::now();
            // Cancels the queries that are running and waits for their snapshots to go away.
            worktrees.set_file_text(host.raw_database_mut(), vfs, edited, Some(text));
            changes.push(applying.elapsed());
            for sender in &snapshots {
                _ = sender.send(host.analysis());
            }
        }
        // The last change may have taken us past the time asked for.
        measured = start.elapsed();
        stop.store(true, Ordering::Relaxed);
        drop(snapshots);
    });

    let done = done.load(Ordering::Relaxed);
    changes.sort();
    let millis = |it: Option<&Duration>| it.map_or(0.0, |it| it.as_secs_f64() * 1000.0);
    println!(
        "churn  {readers:3} readers  change every {interval:5} ms  {:8.0} queries/s  {:6} cancelled  \
         {:5} changes: median {:7.1} ms  max {:7.1} ms",
        done as f64 / measured.as_secs_f64(),
        cancelled.load(Ordering::Relaxed),
        changes.len(),
        millis(changes.get(changes.len() / 2)),
        millis(changes.last()),
    );
}

fn cargo_config() -> CargoConfig {
    CargoConfig { sysroot: Some(RustLibSource::Discover), ..CargoConfig::default() }
}

// The directory is given to git with `-C`.
#[allow(clippy::disallowed_methods)]
fn git(dir: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["-c", "user.name=bench", "-c", "user.email=bench@localhost", "-C", dir])
        .args(args)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn rss_mb() -> i64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
            line.split_whitespace().nth(1)?.parse::<i64>().ok()
        })
        .map_or(0, |kb| kb / 1024)
}

struct Scenario {
    worktrees: Worktrees,
    vfs: Vfs,
    host: AnalysisHost,
    overlay: Overlay,
    roots: Vec<String>,
    base_crates: usize,
}

impl Scenario {
    fn path(&self, in_worktree: &str) -> AbsPathBuf {
        self.overlay.worktree_root.join(in_worktree)
    }

    /// Does `change`, then asks for the diagnostics of `probe` of the worktree as a client that
    /// waits for an answer would, then for those of every file.
    fn step(
        &mut self,
        name: &str,
        probe: &str,
        change: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) {
        let start = Instant::now();
        if let Err(err) = change(self) {
            println!("{name:44} FAILED: {err:#}");
            return;
        }
        let applied = start.elapsed();

        let start = Instant::now();
        let analysis = self.host.analysis();
        let views = self.worktrees.views();
        let in_a_crate = |file| analysis.crates_for(file).is_ok_and(|crates| !crates.is_empty());
        let probed = views
            .file(&self.vfs, &self.path(probe))
            .map(|file| views.analyzed_file(&self.vfs, file, in_a_crate));
        let config = DiagnosticsConfig::test_sample();
        let answer = match probed {
            Some(file) => {
                analysis.full_diagnostics(&config, AssistResolveStrategy::None, file).unwrap();
                format!("{:6.2}s", start.elapsed().as_secs_f64())
            }
            None => "   n/a ".to_owned(),
        };
        drop(analysis);

        let roots: Vec<&String> = self.roots.iter().collect();
        let files = files_of_crates(&self.host, &self.vfs, &roots);
        let everything = diagnostics_pass(&self.host, &files, 16);
        println!(
            "{name:44} applied {:6.2}s  answer {answer}  all files again {:6.2}s  {:+4} crates  {:5} MB",
            applied.as_secs_f64(),
            everything.as_secs_f64(),
            all_crates(self.host.raw_database()).len() as i64 - self.base_crates as i64,
            rss_mb(),
        );
    }

    fn set_text(&mut self, in_worktree: &str, text: String) {
        let path = self.path(in_worktree);
        self.worktrees.set_file_text(
            self.host.raw_database_mut(),
            &mut self.vfs,
            &path,
            Some(text),
        );
    }

    fn reload(&mut self, in_worktree: &str) {
        let path = self.path(in_worktree);
        self.worktrees.reload_file(self.host.raw_database_mut(), &mut self.vfs, &path);
    }

    /// Switches the worktree to `branch` and reports the files that differ, as a watcher would.
    fn checkout(&mut self, branch: &str) -> anyhow::Result<()> {
        let root = self.overlay.worktree_root.to_string();
        let before = git(&root, &["rev-parse", "HEAD"])?;
        git(&root, &["checkout", "-q", branch])?;
        let changed = git(&root, &["diff", "--name-only", before.trim(), "HEAD"])?;
        for file in changed.lines() {
            self.reload(file);
        }
        Ok(())
    }

    /// Loads the workspace of the worktree again after its manifests changed.
    fn manifests_changed(&mut self, manifests: &[&str]) -> anyhow::Result<()> {
        for manifest in manifests {
            self.reload(manifest);
        }
        let workspace = match self.worktrees.workspace_of_copy(&self.overlay) {
            Some(workspace) => workspace,
            None => {
                let manifest = ProjectManifest::discover_single(&self.overlay.worktree_root)?;
                ProjectWorkspace::load(manifest, &cargo_config(), &|_| {})?
            }
        };
        self.worktrees.add(
            self.host.raw_database_mut(),
            &mut self.vfs,
            workspace,
            self.overlay.clone(),
        );
        Ok(())
    }
}

fn scenario(base: &str) -> anyhow::Result<()> {
    let base_root = AbsPathBuf::assert_utf8(std::fs::canonicalize(base)?);
    // Names of our own, so that nothing of the user's is touched.
    let id = std::process::id();
    let (main, feature) = (format!("scn-{id}-main"), format!("scn-{id}-feature"));
    let (main, feature) = (main.as_str(), feature.as_str());
    let worktree_root = base_root.parent().unwrap().join(format!("wt-scn-{id}"));
    anyhow::ensure!(
        !std::path::Path::new(worktree_root.as_str()).exists(),
        "{worktree_root} exists already"
    );
    let worktree = worktree_root.to_string();

    // A worktree at the commit of the base checkout, and a branch that differs from it in a
    // crate in the middle of the dependency graph and in two at its end.
    git(base, &["worktree", "add", "-q", "-b", main, &worktree, "HEAD"])?;
    git(&worktree, &["checkout", "-q", "-b", feature])?;
    let mut touched = vec![
        worktree_root.join("crates/ide-assists/src/lib.rs"),
        worktree_root.join("crates/rust-analyzer/src/lib.rs"),
    ];
    for entry in std::fs::read_dir(worktree_root.join("crates/hir-ty/src"))?.flatten() {
        if entry.path().extension().is_some_and(|it| it == "rs") {
            touched.push(AbsPathBuf::assert_utf8(entry.path()));
        }
    }
    for (idx, file) in touched.iter().enumerate() {
        let text = std::fs::read_to_string(file)?;
        std::fs::write(file, format!("{text}\npub fn on_the_branch_{idx}() {{}}\n"))?;
    }
    git(&worktree, &["commit", "-q", "-am", "branch"])?;
    git(&worktree, &["checkout", "-q", main])?;
    println!("the branch differs in {} files", touched.len());

    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 16,
        proc_macro_processes: 1,
    };
    let start = Instant::now();
    let manifest = ProjectManifest::discover_single(&base_root)?;
    let workspace = ProjectWorkspace::load(manifest, &cargo_config(), &|_| {})?;
    let (worktrees, db, vfs) = Worktrees::load(workspace, &FxHashMap::default(), &load_config)?;
    let host = AnalysisHost::with_database(db);
    let overlay = Overlay { worktree_root, base_root };
    let mut scn = Scenario {
        base_crates: all_crates(host.raw_database()).len(),
        worktrees,
        vfs,
        host,
        overlay,
        roots: vec![base.to_owned(), worktree.clone()],
    };
    let roots: Vec<&String> = scn.roots.iter().collect();
    let files = files_of_crates(&scn.host, &scn.vfs, &roots);
    diagnostics_pass(&scn.host, &files, 16);
    println!(
        "base checkout loaded and analyzed in {:.1}s: {} crates, {} MB",
        start.elapsed().as_secs_f64(),
        scn.base_crates,
        rss_mb()
    );

    let leaf = "crates/rust-analyzer/src/lib.rs";
    let core = "crates/stdx/src/lib.rs";
    let far = "crates/ide/src/lib.rs";
    let edited = |scn: &Scenario, file: &str, name: &str| -> anyhow::Result<String> {
        Ok(format!("{}\npub fn {name}() {{}}\n", std::fs::read_to_string(scn.path(file))?))
    };

    scn.step("worktree added", leaf, |scn| scn.manifests_changed(&[]));
    scn.step("edit in a crate nothing depends on", leaf, |scn| {
        let text = edited(scn, leaf, "edited")?;
        scn.set_text(leaf, text);
        Ok(())
    });
    scn.step("second edit of the same file", leaf, |scn| {
        let text = edited(scn, leaf, "edited_again")?;
        scn.set_text(leaf, text);
        Ok(())
    });
    scn.step("edit in a crate 35 crates depend on", far, |scn| {
        let text = edited(scn, core, "edited")?;
        scn.set_text(core, text);
        Ok(())
    });
    scn.step("second edit of the same file", far, |scn| {
        let text = edited(scn, core, "edited_again")?;
        scn.set_text(core, text);
        Ok(())
    });
    scn.step("both edits undone", far, |scn| {
        scn.reload(leaf);
        scn.reload(core);
        Ok(())
    });
    scn.step("git checkout of the branch", far, |scn| scn.checkout(feature));
    scn.step("git checkout back", far, |scn| scn.checkout(main));

    // Another version of a dependency, one that the lock file has already: first for one crate,
    // then for every crate of the workspace.
    let replace = |scn: &Scenario, file: &str, from: &str, to: &str| -> anyhow::Result<()> {
        let text = std::fs::read_to_string(scn.path(file))?;
        anyhow::ensure!(text.contains(from), "{file} has no `{from}`");
        std::fs::write(scn.path(file), text.replace(from, to))?;
        Ok(())
    };
    let one = "crates/ide-ssr/Cargo.toml";
    scn.step("other version of a dependency, one crate", "crates/ide-ssr/src/lib.rs", |scn| {
        replace(scn, one, "itertools.workspace = true", "itertools = \"0.13.0\"")?;
        scn.manifests_changed(&[one, "Cargo.lock"])
    });
    scn.step("... and back", "crates/ide-ssr/src/lib.rs", |scn| {
        git(&scn.overlay.worktree_root.to_string(), &["checkout", "-q", "--", "."])?;
        scn.manifests_changed(&[one, "Cargo.lock"])
    });
    scn.step("other version of a dependency, all crates", far, |scn| {
        replace(scn, "Cargo.toml", "rustc-hash = \"2.1.1\"", "rustc-hash = \"1.1.0\"")?;
        scn.manifests_changed(&["Cargo.toml", "Cargo.lock"])
    });
    scn.step("... and back", far, |scn| {
        git(&scn.overlay.worktree_root.to_string(), &["checkout", "-q", "--", "."])?;
        scn.manifests_changed(&["Cargo.toml", "Cargo.lock"])
    });

    // Does what is left behind by crates that are not there anymore add up?
    for round in 0..6 {
        scn.step(&format!("round {round}: edit in the crate 35 depend on"), far, |scn| {
            let text = edited(scn, core, &format!("round_{round}"))?;
            scn.set_text(core, text);
            Ok(())
        });
        scn.step(&format!("round {round}: undone"), far, |scn| {
            scn.reload(core);
            Ok(())
        });
    }
    let start = Instant::now();
    scn.host.trigger_garbage_collection();
    println!("garbage collection {:.2}s  {} MB", start.elapsed().as_secs_f64(), rss_mb());

    drop(scn);
    // The worktree and the branches are ours, and what we changed in it is reverted.
    git(base, &["worktree", "remove", &worktree])?;
    git(base, &["branch", "-D", main, feature])?;
    Ok(())
}

/// A function is added to `stdx` of a checkout whose return type tells which checkout it is,
/// and a use of it with another type to `ide`: the type in the diagnostic of that use is the one
/// of the checkout exactly if the checkout sees its own `stdx` and nobody else's.
fn isolation(base: &str, copies: usize, changes: usize) -> anyhow::Result<()> {
    const STDX: &str = "crates/stdx/src/lib.rs";
    const IDE: &str = "crates/ide/src/lib.rs";
    let base_root = AbsPathBuf::assert_utf8(std::fs::canonicalize(base)?);
    let id = std::process::id();
    let mut roots = vec![base_root.clone()];
    for copy in 0..copies {
        let root = base_root.parent().unwrap().join(format!("wt-iso-{id}-{copy}"));
        anyhow::ensure!(!std::path::Path::new(root.as_str()).exists(), "{root} exists already");
        git(
            base,
            &["worktree", "add", "-q", "-b", &format!("iso-{id}-{copy}"), root.as_str(), "HEAD"],
        )?;
        roots.push(root);
    }
    let stdx_text = std::fs::read_to_string(base_root.join(STDX))?;
    let ide_text = std::fs::read_to_string(base_root.join(IDE))?;

    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 16,
        proc_macro_processes: 1,
    };
    let manifest = ProjectManifest::discover_single(&base_root)?;
    let workspace = ProjectWorkspace::load(manifest, &cargo_config(), &|_| {})?;
    let (mut worktrees, db, mut vfs) =
        Worktrees::load(workspace, &FxHashMap::default(), &load_config)?;
    let mut host = AnalysisHost::with_database(db);
    let root_names: Vec<String> = roots.iter().map(|root| root.to_string()).collect();
    let root_names: Vec<&String> = root_names.iter().collect();
    let config = DiagnosticsConfig::test_sample();

    // The diagnostics of every file of the base checkout, to compare with at the end.
    let fingerprint = |host: &AnalysisHost, vfs: &Vfs| -> Vec<(String, Vec<String>)> {
        let analysis = host.analysis();
        let mut res: Vec<_> = files_of_crates(host, vfs, &root_names[..1])
            .into_iter()
            .map(|file| {
                let diagnostics =
                    analysis.full_diagnostics(&config, AssistResolveStrategy::None, file).unwrap();
                let messages = diagnostics.into_iter().map(|it| it.message).collect();
                (vfs.file_path(file).to_string(), messages)
            })
            .collect();
        res.sort();
        res
    };
    let analyze_everything = |host: &AnalysisHost, vfs: &Vfs| {
        let files = files_of_crates(host, vfs, &root_names);
        diagnostics_pass(host, &files, 16);
    };
    analyze_everything(&host, &vfs);
    let before = fingerprint(&host, &vfs);
    let base_crates = all_crates(host.raw_database()).len();
    let base_rss = rss_mb();
    println!("base checkout analyzed: {base_crates} crates, {base_rss} MB");

    // How much of the base checkout alone comes back, to compare with.
    if let Some(rounds) = std::env::var("GC_FIRST").ok().and_then(|it| it.parse::<usize>().ok()) {
        for round in 0..rounds {
            worktrees.collect_garbage(host.raw_database_mut(), &mut vfs);
            host.trigger_garbage_collection();
            give_back_free_memory();
            let collected = rss_mb();
            analyze_everything(&host, &vfs);
            println!(
                "round {round}: {collected} MB after a collection, {} MB analyzed again",
                rss_mb()
            );
        }
        // With everything dropped, what is left is what nothing frees: a heap profiler tells
        // where it was allocated.
        drop((worktrees, host, vfs));
        // SAFETY: There is no database anymore that could refer to a type.
        unsafe { hir::collect_ty_garbage() };
        give_back_free_memory();
        println!("everything dropped: {} MB left", rss_mb());
        return Ok(());
    }

    for root in &roots[1..] {
        let overlay = Overlay { worktree_root: root.clone(), base_root: base_root.clone() };
        let workspace = worktrees
            .workspace_of_copy(&overlay)
            .ok_or_else(|| anyhow::format_err!("the workspace of a copy has to be loaded"))?;
        worktrees.add(host.raw_database_mut(), &mut vfs, workspace, overlay);
    }
    analyze_everything(&host, &vfs);
    println!(
        "{copies} worktrees added: {:+} crates, {:+} MB",
        all_crates(host.raw_database()).len() as i64 - base_crates as i64,
        rss_mb() - base_rss
    );

    // The size of the array the function of a checkout returns, `None` without the function.
    let mut sizes: Vec<Option<usize>> = vec![None; roots.len()];
    let mut failures = 0;
    let mut check = |host: &AnalysisHost,
                     vfs: &Vfs,
                     worktrees: &Worktrees,
                     sizes: &[Option<usize>],
                     change: usize| {
        let analysis = host.analysis();
        let views = worktrees.views();
        for (idx, root) in roots.iter().enumerate() {
            let in_a_crate =
                |file| analysis.crates_for(file).is_ok_and(|crates| !crates.is_empty());
            let file = views.file(vfs, &root.join(IDE)).expect("the file is loaded");
            let analyzed = views.analyzed_file(vfs, file, in_a_crate);
            // The file of the base checkout is analyzed for a worktree exactly while the two
            // have the same texts.
            let base_file = views.file(vfs, &roots[0].join(IDE)).expect("the file is loaded");
            let stands_in = idx != 0 && analyzed == base_file;
            _ = file;
            let seen: Vec<String> = analysis
                .full_diagnostics(&config, AssistResolveStrategy::None, analyzed)
                .unwrap()
                .into_iter()
                .map(|it| it.message)
                .filter(|message| message.contains("[u8; 99]"))
                .collect();
            let ok = match sizes[idx] {
                // Its own function and no other.
                Some(size) => seen.len() == 1 && seen[0].contains(&format!("[u8; {size}]")),
                // No use of the function, whatever the base checkout has.
                None => seen.is_empty(),
            } && (idx == 0 || stands_in == (sizes[idx] == sizes[0]));
            if !ok {
                failures += 1;
                println!(
                    "change {change}: checkout {idx} expected {:?}, sees {seen:?} (base has {:?}, stands in: {stands_in})",
                    sizes[idx], sizes[0]
                );
            }
        }
        failures
    };

    let mut random = {
        let mut state = 0x2545f4914f6cdd1du64;
        move |below: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % below as u64) as usize
        }
    };
    let (mut rss, mut largest_step, mut peak) = (rss_mb(), 0, 0);
    for change in 0..changes {
        // The base checkout is changed less often than the worktrees.
        let idx = if random(8) == 0 { 0 } else { 1 + random(copies) };
        let (stdx, ide) = (roots[idx].join(STDX), roots[idx].join(IDE));
        let db = host.raw_database_mut();
        if sizes[idx].is_some() && random(3) == 0 {
            worktrees.reload_file(db, &mut vfs, &stdx);
            worktrees.reload_file(db, &mut vfs, &ide);
            sizes[idx] = None;
        } else {
            let size = 1 + random(90);
            let function =
                format!("{stdx_text}\npub fn iso_marker() -> [u8; {size}] {{ [0; {size}] }}\n");
            let usage = format!(
                "{ide_text}\npub fn iso_use() {{ let _: [u8; 99] = stdx::iso_marker(); }}\n"
            );
            worktrees.set_file_text(db, &mut vfs, &stdx, Some(function));
            worktrees.set_file_text(db, &mut vfs, &ide, Some(usage));
            sizes[idx] = Some(size);
        }
        let failed = check(&host, &vfs, &worktrees, &sizes, change);
        // Everything is analyzed, as with clients that look at every file; with `LAZY`, only
        // the file that was checked, as with clients that look at what they work on.
        if std::env::var_os("LAZY").is_none() {
            analyze_everything(&host, &vfs);
        }
        let now = rss_mb();
        if now - rss > 300 {
            println!(
                "change {change}: {:+} MB by a change to {}, {:+} crates now",
                now - rss,
                if idx == 0 { "the base checkout".to_owned() } else { format!("worktree {idx}") },
                all_crates(host.raw_database()).len() as i64 - base_crates as i64,
            );
        }
        (largest_step, peak) = (largest_step.max(now - rss), peak.max(now));
        rss = now;
        if (change + 1) % 25 == 0 {
            println!(
                "after {:4} changes: {:2} checkouts changed  {:+4} crates  {rss:5} MB  largest step {largest_step:+} MB  {failed} failures",
                change + 1,
                sizes.iter().flatten().count(),
                all_crates(host.raw_database()).len() as i64 - base_crates as i64,
            );
        }
    }

    for (idx, root) in roots.iter().enumerate() {
        if sizes[idx].take().is_some() {
            worktrees.reload_file(host.raw_database_mut(), &mut vfs, &root.join(STDX));
            worktrees.reload_file(host.raw_database_mut(), &mut vfs, &root.join(IDE));
        }
    }
    let failed = check(&host, &vfs, &worktrees, &sizes, changes);
    analyze_everything(&host, &vfs);
    println!(
        "everything undone: {:+} crates, {} MB, peak {peak} MB; diagnostics of the base checkout {}; {failed} failures",
        all_crates(host.raw_database()).len() as i64 - base_crates as i64,
        rss_mb(),
        if fingerprint(&host, &vfs) == before { "as before" } else { "DIFFER" },
    );
    for root in &roots[1..] {
        worktrees.remove(host.raw_database_mut(), &mut vfs, root);
    }
    host.trigger_garbage_collection();
    println!(
        "worktrees removed: {:+} crates, {} MB, {} MB of it in use by the program",
        all_crates(host.raw_database()).len() as i64 - base_crates as i64,
        rss_mb(),
        heap_in_use_mb()
    );
    give_back_free_memory();
    println!("what the allocator holds given back to the system: {} MB left", rss_mb());
    host.trigger_garbage_collection();
    give_back_free_memory();
    println!("types collected, database kept: {} MB left", rss_mb());
    let start = Instant::now();
    worktrees.collect_garbage(host.raw_database_mut(), &mut vfs);
    give_back_free_memory();
    println!("garbage collected in {:.2}s: {} MB left", start.elapsed().as_secs_f64(), rss_mb());
    let start = Instant::now();
    // The types that were computed are kept outside of the database.
    host.trigger_garbage_collection();
    give_back_free_memory();
    println!("types collected in {:.2}s: {} MB left", start.elapsed().as_secs_f64(), rss_mb());
    let start = Instant::now();
    analyze_everything(&host, &vfs);
    println!(
        "base checkout analyzed again in {:.1}s: {} crates, {} MB; its diagnostics {}",
        start.elapsed().as_secs_f64(),
        all_crates(host.raw_database()).len(),
        rss_mb(),
        if fingerprint(&host, &vfs) == before { "as before" } else { "DIFFER" },
    );

    drop((worktrees, host));
    for (copy, root) in roots[1..].iter().enumerate() {
        git(base, &["worktree", "remove", root.as_str()])?;
        git(base, &["branch", "-D", &format!("iso-{id}-{copy}")])?;
    }
    Ok(())
}

/// Asks the allocator to return to the system what the program has freed.
fn give_back_free_memory() {
    // SAFETY: Neither has preconditions; tcmalloc's function is there if that is preloaded.
    unsafe {
        let release = dlsym(std::ptr::null_mut(), c"MallocExtension_ReleaseFreeMemory".as_ptr());
        if release.is_null() {
            malloc_trim(0);
        } else {
            std::mem::transmute::<*mut std::ffi::c_void, extern "C" fn()>(release)();
        }
    }
}

unsafe extern "C" {
    fn dlsym(handle: *mut std::ffi::c_void, name: *const std::ffi::c_char)
    -> *mut std::ffi::c_void;
    fn malloc_trim(pad: usize) -> i32;
    fn mallinfo2() -> MallInfo2;
}

#[repr(C)]
struct MallInfo2 {
    arena: usize,
    ordblks: usize,
    smblks: usize,
    hblks: usize,
    hblkhd: usize,
    usmblks: usize,
    fsmblks: usize,
    uordblks: usize,
    fordblks: usize,
    keepcost: usize,
}

/// What the program has allocated and not freed, as opposed to what the allocator holds.
fn heap_in_use_mb() -> usize {
    // SAFETY: Has no preconditions.
    let info = unsafe { mallinfo2() };
    (info.uordblks + info.hblkhd) / (1024 * 1024)
}
