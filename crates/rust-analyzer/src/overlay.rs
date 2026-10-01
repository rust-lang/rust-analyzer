//! Sharing of analysis between a checkout and the git worktrees created from it, see
//! [`load_cargo::worktree`].

use std::fs;

use load_cargo::worktree::worktree_of;
pub(crate) use load_cargo::worktree::{
    DiskCache, Overlay, OverlayCrates, PulledInFile, SourceRoots, Sources, crate_graph,
    find_overlays, pulled_in_files, same_sources, shared_base_file,
};
use project_model::ProjectManifest;

use crate::config::LinkedProject;

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
