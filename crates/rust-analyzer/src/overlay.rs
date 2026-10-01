//! Sharing of analysis between a checkout and the git worktrees created from it.
//!
//! A worktree is a second copy of mostly the same sources at another path. Loading it as an
//! ordinary workspace analyzes every local crate a second time. Instead, a crate of the worktree
//! whose package is identical to the one in the base checkout, and whose dependencies are shared
//! as well, is replaced by the crate of the base checkout when the crate graph is built. Only the
//! packages that differ, and whatever depends on them, are analyzed for the worktree.

use std::fs;

use ide_db::{
    FxHashMap,
    base_db::{CrateBuilder, CrateBuilderId, CrateGraphBuilder, SourceRoot},
};
use load_cargo::SourceRootConfig;
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

/// Whether the source root of `worktree_file` has the same files with the same contents as the
/// source root of `base_file`.
pub(crate) fn same_sources(
    vfs: &Vfs,
    roots: &SourceRoots,
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
        base_file.is_some_and(|base_file| vfs.content_hash(file) == vfs.content_hash(base_file))
    })
}

/// Picks the crate of the base checkout that can stand in for the worktree's `krate`.
pub(crate) fn base_crate(
    vfs: &Vfs,
    roots: &SourceRoots,
    overlay: &Overlay,
    overlay_crates: &mut OverlayCrates,
    graph: &CrateGraphBuilder,
    krate: &CrateBuilder,
) -> Option<CrateBuilderId> {
    let worktree_file = krate.basic.root_file_id;
    let base_file = match overlay.to_base(vfs.file_path(worktree_file).as_path()?) {
        Some(base_path) => {
            let (base_file, _) = vfs.file_id(&VfsPath::from(base_path))?;
            let same_sources = same_sources(vfs, roots, overlay, worktree_file, base_file);
            overlay_crates.insert((worktree_file, base_file), same_sources);
            if !same_sources {
                return None;
            }
            base_file
        }
        // A library: the very same files, used by both.
        None => worktree_file,
    };
    graph.iter().find(|&id| {
        let base_crate = &graph[id];
        base_crate.basic.root_file_id == base_file
            && base_crate.eq_modulo_location(
                krate,
                overlay.base_root.as_str(),
                overlay.worktree_root.as_str(),
            )
    })
}

/// For the crates of worktrees that have a counterpart in the base checkout, as the root files
/// of both, whether the sources of the two were the same when the crate graph was built.
pub(crate) type OverlayCrates = FxHashMap<(FileId, FileId), bool>;
