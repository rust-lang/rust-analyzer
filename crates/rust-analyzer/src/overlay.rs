//! Sharing of analysis between a checkout and the git worktrees created from it.
//!
//! A worktree is a second copy of mostly the same sources at another path. Loading it as an
//! ordinary workspace analyzes every local crate a second time. Instead, a crate of the worktree
//! whose package is identical to the one in the base checkout, and whose dependencies are shared
//! as well, is replaced by the crate of the base checkout when the crate graph is built. Only the
//! packages that differ, and whatever depends on them, are analyzed for the worktree.

use std::fs;

use ide_db::{
    FxHashMap, FxHashSet,
    base_db::{CrateBuilder, CrateBuilderId, CrateGraphBuilder, SourceRoot},
};
use load_cargo::SourceRootConfig;
use paths::Utf8Path;
use project_model::{ProjectManifest, ProjectWorkspace};
use vfs::{AbsPath, AbsPathBuf, FileId, Vfs, VfsPath};

use crate::config::LinkedProject;

/// A workspace that lives in a git worktree, together with its counterpart in the checkout the
/// worktree was created from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Overlay {
    pub(crate) worktree_root: AbsPathBuf,
    pub(crate) base_root: AbsPathBuf,
}

impl Overlay {
    /// The path in the base checkout that corresponds to `path` in the worktree.
    pub(crate) fn to_base(&self, path: &AbsPath) -> Option<AbsPathBuf> {
        Some(self.base_root.join(path.strip_prefix(&self.worktree_root)?))
    }

    /// The path in the worktree that corresponds to `path` in the base checkout.
    pub(crate) fn to_worktree(&self, path: &AbsPath) -> Option<AbsPathBuf> {
        Some(self.worktree_root.join(path.strip_prefix(&self.base_root)?))
    }
}

/// Finds the git worktree that contains `path` and the checkout it was created from.
fn worktree_of(path: &AbsPath) -> Option<Overlay> {
    let worktree_root = std::iter::successors(Some(path), |it| it.parent())
        .find(|dir| fs::metadata(dir.join(".git")).is_ok())?;
    // In a worktree `.git` is a file that points into the git directory of the base checkout:
    // `gitdir: /base/.git/worktrees/<name>`.
    let dot_git = fs::read_to_string(worktree_root.join(".git")).ok()?;
    let git_dir = dot_git.lines().find_map(|line| line.strip_prefix("gitdir:"))?.trim();
    let git_dir = worktree_root.absolutize(git_dir);
    let worktrees_dir = git_dir.parent()?;
    let common_dir = worktrees_dir.parent()?;
    if worktrees_dir.file_name() != Some("worktrees") || common_dir.file_name() != Some(".git") {
        return None;
    }
    let base_root = common_dir.parent()?;
    Some(Overlay { worktree_root: worktree_root.to_path_buf(), base_root: base_root.to_path_buf() })
}

/// For each workspace, the overlay it forms over another loaded workspace, if any.
pub(crate) fn find_overlays(workspaces: &[ProjectWorkspace]) -> Vec<Option<Overlay>> {
    workspaces
        .iter()
        .map(|ws| {
            let overlay = worktree_of(ws.workspace_root())?;
            let base_ws_root = overlay.to_base(ws.workspace_root())?;
            workspaces
                .iter()
                .any(|base| base.workspace_root() == base_ws_root.as_path())
                .then_some(overlay)
        })
        .collect()
}

/// The projects in the base checkouts of the worktrees that `projects` are in, unless they are
/// among `projects` already.
pub(crate) fn base_checkouts(projects: &[LinkedProject]) -> Vec<LinkedProject> {
    let manifest = |project: &LinkedProject| match project {
        LinkedProject::ProjectManifest(manifest) => Some(manifest.manifest_path().clone()),
        LinkedProject::InlineProjectJson(_) => None,
    };
    let mut known: Vec<_> = projects.iter().filter_map(manifest).collect();
    let mut res = Vec::new();
    for project in projects {
        let Some(manifest) = manifest(project) else { continue };
        let Some(base_manifest) =
            worktree_of(manifest.parent()).and_then(|overlay| overlay.to_base(&manifest))
        else {
            continue;
        };
        if known.iter().any(|it| **it == *base_manifest) || fs::metadata(&base_manifest).is_err() {
            continue;
        }
        if let Ok(base_project) = ProjectManifest::from_manifest_file(base_manifest) {
            known.push(base_project.manifest_path().clone());
            res.push(base_project.into());
        }
    }
    res
}

/// The partition of the files into source roots, as of the moment the crate graph is built.
pub(crate) struct SourceRoots {
    roots: Vec<SourceRoot>,
    root_of: FxHashMap<FileId, usize>,
}

impl SourceRoots {
    pub(crate) fn new(config: &SourceRootConfig, vfs: &Vfs) -> SourceRoots {
        let roots = config.partition(vfs);
        let root_of = roots
            .iter()
            .enumerate()
            .flat_map(|(idx, root)| root.iter().map(move |file| (file, idx)))
            .collect();
        SourceRoots { roots, root_of }
    }

    fn of(&self, file: FileId) -> Option<&SourceRoot> {
        Some(&self.roots[*self.root_of.get(&file)?])
    }

    /// Whether the two files are in the same source root.
    pub(crate) fn in_same_root(&self, file: FileId, other: FileId) -> bool {
        match (self.root_of.get(&file), self.root_of.get(&other)) {
            (Some(root), Some(other_root)) => root == other_root,
            _ => false,
        }
    }
}

/// Whether the text of a source file may pull in sources from outside of its package, which
/// comparing the packages file by file does not cover.
pub(crate) fn reaches_outside(text: &str) -> bool {
    (text.contains("#[path") || text.contains("include"))
        && (text.contains("../") || text.contains("= \"/") || text.contains("(\"/"))
}

/// Whether the source root of `worktree_file` has the same files with the same contents as the
/// source root of `base_file`, and none of them reaches for sources outside of it.
pub(crate) fn same_sources(
    vfs: &Vfs,
    roots: &SourceRoots,
    reaching_outside: &FxHashSet<FileId>,
    overlay: &Overlay,
    worktree_file: FileId,
    base_file: FileId,
) -> bool {
    let (Some(worktree_root), Some(base_root)) = (roots.of(worktree_file), roots.of(base_file))
    else {
        return false;
    };
    if worktree_root.iter().count() != base_root.iter().count() {
        return false;
    }
    worktree_root.iter().all(|file| {
        let base_file = worktree_root
            .path_for_file(&file)
            .and_then(|path| overlay.to_base(path.as_path()?))
            .and_then(|path| base_root.file_for_path(&VfsPath::from(path)).copied());
        base_file.is_some_and(|base_file| {
            vfs.content_hash(file) == vfs.content_hash(base_file)
                && !reaching_outside.contains(&file)
        })
    })
}

/// Whether the two directories have the same files with the same contents.
fn same_dir(dir: &Utf8Path, other: &Utf8Path) -> bool {
    let entries = |dir: &Utf8Path| -> Option<Vec<(String, bool)>> {
        let mut entries = fs::read_dir(dir)
            .ok()?
            .map(|entry| {
                let entry = entry.ok()?;
                Some((entry.file_name().into_string().ok()?, entry.file_type().ok()?.is_dir()))
            })
            .collect::<Option<Vec<_>>>()?;
        entries.sort();
        Some(entries)
    };
    let (Some(entries), Some(other_entries)) = (entries(dir), entries(other)) else {
        return false;
    };
    entries == other_entries
        && entries.iter().all(|(name, is_dir)| {
            let (path, other_path) = (dir.join(name), other.join(name));
            if *is_dir {
                same_dir(&path, &other_path)
            } else {
                matches!((fs::read(path), fs::read(other_path)), (Ok(it), Ok(other)) if it == other)
            }
        })
}

/// Whether what the build scripts of the two crates generated is the same.
fn same_build_script_output(krate: &CrateBuilder, base_crate: &CrateBuilder) -> bool {
    match (krate.env.get("OUT_DIR"), base_crate.env.get("OUT_DIR")) {
        (None, None) => true,
        (Some(out_dir), Some(base_out_dir)) => {
            out_dir == base_out_dir
                || same_dir(Utf8Path::new(&out_dir), Utf8Path::new(&base_out_dir))
        }
        _ => false,
    }
}

/// Picks the crate of the base checkout that can stand in for the worktree's `krate`.
pub(crate) fn base_crate(
    vfs: &Vfs,
    roots: &SourceRoots,
    reaching_outside: &FxHashSet<FileId>,
    overlay: &Overlay,
    overlay_crates: &mut OverlayCrates,
    graph: &CrateGraphBuilder,
    krate: &CrateBuilder,
) -> Option<CrateBuilderId> {
    let worktree_file = krate.basic.root_file_id;
    let base_file = match overlay.to_base(vfs.file_path(worktree_file).as_path()?) {
        Some(base_path) => {
            let (base_file, _) = vfs.file_id(&VfsPath::from(base_path))?;
            let same_sources =
                same_sources(vfs, roots, reaching_outside, overlay, worktree_file, base_file);
            overlay_crates
                .insert((worktree_file, base_file), OverlayCrate { same_sources, shared: false });
            if !same_sources {
                return None;
            }
            base_file
        }
        // A library: the very same files, used by both.
        None => worktree_file,
    };
    let base_crate = graph.iter().find(|&id| {
        let base_crate = &graph[id];
        base_crate.basic.root_file_id == base_file
            && base_crate.eq_modulo_location(
                krate,
                overlay.base_root.as_str(),
                overlay.worktree_root.as_str(),
            )
            && same_build_script_output(krate, base_crate)
    })?;
    if let Some(overlay_crate) = overlay_crates.get_mut(&(worktree_file, base_file)) {
        overlay_crate.shared = true;
    }
    Some(base_crate)
}

/// The file of the base checkout that is analyzed in place of the worktree's `file`: the two are
/// the same, and all crates of the package of `file` are shared with the base checkout.
pub(crate) fn shared_base_file(
    vfs: &Vfs,
    roots: &SourceRoots,
    overlays: &[Overlay],
    overlay_crates: &OverlayCrates,
    file: FileId,
) -> Option<FileId> {
    let path = vfs.file_path(file).as_path()?;
    let overlay = overlays.iter().find(|it| path.starts_with(&it.worktree_root))?;
    let (base_file, _) = vfs.file_id(&VfsPath::from(overlay.to_base(path)?))?;
    if vfs.content_hash(file) != vfs.content_hash(base_file) {
        return None;
    }
    let mut crates_of_package = overlay_crates
        .iter()
        .filter(|&(&(worktree_root_file, _), _)| roots.in_same_root(worktree_root_file, file))
        .peekable();
    crates_of_package.peek()?;
    crates_of_package.all(|(_, krate)| krate.shared).then_some(base_file)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OverlayCrate {
    /// Whether the sources of the crate were the same as in the base checkout when the crate
    /// graph was built.
    pub(crate) same_sources: bool,
    /// Whether the crate of the base checkout stands in for the crate.
    pub(crate) shared: bool,
}

/// The crates of worktrees that have a counterpart in the base checkout, by the root files of
/// both.
pub(crate) type OverlayCrates = FxHashMap<(FileId, FileId), OverlayCrate>;

#[cfg(test)]
mod tests {
    use super::reaches_outside;

    #[test]
    fn detects_sources_outside_of_the_package() {
        assert!(reaches_outside("#[path = \"../shared.rs\"]\nmod shared;"));
        assert!(reaches_outside("include!(\"../../gen.rs\");"));
        assert!(reaches_outside("const S: &str = include_str!(\"/etc/hosts\");"));
        assert!(!reaches_outside("#[path = \"imp/unix.rs\"]\nmod imp;"));
        assert!(!reaches_outside("mod a;\nuse super::b;"));
    }
}
