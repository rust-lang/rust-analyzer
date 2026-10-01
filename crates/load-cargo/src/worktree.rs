//! Sharing of analysis between a checkout and the git worktrees created from it.
//!
//! A worktree is a second copy of mostly the same sources at another path. Loading it as an
//! ordinary workspace analyzes every local crate a second time. Instead, a crate of the worktree
//! whose package is identical to the one in the base checkout, and whose dependencies are shared
//! as well, is replaced by the crate of the base checkout when the crate graph is built. Only the
//! packages that differ, and whatever depends on them, are analyzed for the worktree.

use std::{cell::RefCell, fs};

use ide_db::{
    FxHashMap, FxHashSet,
    base_db::{CrateBuilder, CrateBuilderId, CrateGraphBuilder, ProcMacroPaths, SourceRoot},
};
use parser::{Edition, LexedStr, SyntaxKind};
use paths::Utf8Path;
use project_model::ProjectWorkspace;
use rustc_hash::FxHasher;
use stdx::hash_once;
use vfs::{AbsPath, AbsPathBuf, FileId, Vfs, VfsPath};

use crate::SourceRootConfig;

/// A workspace that lives in a git worktree, together with its counterpart in the checkout the
/// worktree was created from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Overlay {
    pub worktree_root: AbsPathBuf,
    pub base_root: AbsPathBuf,
}

impl Overlay {
    /// The path in the base checkout that corresponds to `path` in the worktree.
    pub fn to_base(&self, path: &AbsPath) -> Option<AbsPathBuf> {
        Some(self.base_root.join(path.strip_prefix(&self.worktree_root)?))
    }

    /// The path in the worktree that corresponds to `path` in the base checkout.
    pub fn to_worktree(&self, path: &AbsPath) -> Option<AbsPathBuf> {
        Some(self.worktree_root.join(path.strip_prefix(&self.base_root)?))
    }
}

/// Finds the git worktree that contains `path` and the checkout it was created from.
pub fn worktree_of(path: &AbsPath) -> Option<Overlay> {
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
pub fn find_overlays(workspaces: &[ProjectWorkspace]) -> Vec<Option<Overlay>> {
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

/// The partition of the files into source roots, as of the moment the crate graph is built.
#[derive(Default)]
pub struct SourceRoots {
    roots: Vec<SourceRoot>,
    root_of: FxHashMap<FileId, usize>,
}

impl SourceRoots {
    pub fn new(config: &SourceRootConfig, vfs: &Vfs) -> SourceRoots {
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

    /// Whether a file in the source root of `file` pulls in other files by path.
    pub fn pulls_in_files(
        &self,
        pulled_in_files: &FxHashMap<FileId, Vec<PulledInFile>>,
        file: FileId,
    ) -> bool {
        self.of(file)
            .is_some_and(|root| root.iter().any(|file| pulled_in_files.contains_key(&file)))
    }

    /// Whether the two files are in the same source root.
    pub fn in_same_root(&self, file: FileId, other: FileId) -> bool {
        match (self.root_of.get(&file), self.root_of.get(&other)) {
            (Some(root), Some(other_root)) => root == other_root,
            _ => false,
        }
    }
}

/// A file that a source file pulls in by path, with `#[path]` or one of the `include!` macros.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PulledInFile {
    /// At the path, relative to the directory of the source file.
    Relative(String),
    /// Somewhere we cannot tell: an absolute or a computed path.
    Unknown,
}

/// The files that the text of a source file pulls in by path.
///
/// Comparing two packages file by file only covers them if they are part of the package.
pub fn pulled_in_files(text: &str) -> Vec<PulledInFile> {
    // Lexing every file is wasteful, most do not even come close.
    if !(text.contains("path") || text.contains("include")) {
        return Vec::new();
    }
    let lexed = LexedStr::new(Edition::CURRENT, text);
    let tokens: Vec<(SyntaxKind, &str)> = (0..lexed.len())
        .map(|idx| (lexed.kind(idx), lexed.text(idx)))
        .filter(|(kind, _)| !kind.is_trivia())
        .collect();
    let literal = |text: &str| {
        let path = text.trim_start_matches(['r', '#']).trim_end_matches('#');
        let path = path.strip_prefix('"')?.strip_suffix('"')?;
        if path.starts_with('/') || path.contains('\\') {
            return None;
        }
        Some(PulledInFile::Relative(path.to_owned()))
    };
    let mut res = Vec::new();
    let mut brace_depth = 0u32;
    // How deep in brackets we are inside of an attribute, 0 outside of attributes.
    let mut attr_depth = 0u32;
    for (idx, &(kind, text)) in tokens.iter().enumerate() {
        match kind {
            SyntaxKind::L_CURLY => brace_depth += 1,
            SyntaxKind::R_CURLY => brace_depth = brace_depth.saturating_sub(1),
            SyntaxKind::L_BRACK if attr_depth > 0 => attr_depth += 1,
            SyntaxKind::L_BRACK => {
                let before = |n: usize| idx.checked_sub(n).map(|idx| tokens[idx].0);
                let is_attr = before(1) == Some(SyntaxKind::POUND)
                    || (before(1) == Some(SyntaxKind::BANG)
                        && before(2) == Some(SyntaxKind::POUND));
                attr_depth = is_attr as u32;
            }
            SyntaxKind::R_BRACK => attr_depth = attr_depth.saturating_sub(1),
            _ => (),
        }
        if kind != SyntaxKind::IDENT {
            continue;
        }
        let before = |n: usize| idx.checked_sub(n).map(|idx| tokens[idx].0);
        let after = |n: usize| tokens.get(idx + n).copied();
        // `#[path = ".."]`, or `#[cfg_attr(unix, path = "..")]`
        let is_path_attr = text == "path"
            && attr_depth > 0
            && matches!(
                before(1),
                Some(SyntaxKind::L_BRACK | SyntaxKind::COMMA | SyntaxKind::L_PAREN)
            )
            && after(1).map(|it| it.0) == Some(SyntaxKind::EQ);
        let is_include = matches!(text, "include" | "include_str" | "include_bytes")
            && after(1).map(|it| it.0) == Some(SyntaxKind::BANG);
        if !is_path_attr && !is_include {
            continue;
        }
        res.push(match after(2) {
            // Inside of an inline module the path is relative to a directory named after it,
            // which we do not track.
            Some((SyntaxKind::STRING, _)) if is_path_attr && brace_depth > 0 => {
                PulledInFile::Unknown
            }
            Some((SyntaxKind::STRING, text)) => literal(text).unwrap_or(PulledInFile::Unknown),
            // `include!("path")` has the delimiter first
            _ => match after(3) {
                Some((SyntaxKind::STRING, text)) if is_include => {
                    literal(text).unwrap_or(PulledInFile::Unknown)
                }
                // What a build script generated is compared separately.
                _ if tokens[idx..].iter().take(12).any(|&(_, text)| text == "\"OUT_DIR\"") => {
                    continue;
                }
                _ => PulledInFile::Unknown,
            },
        });
    }
    res
}

/// Whether two files that are not loaded have the same contents, by the paths of both.
///
/// Such files are not watched, so this is only as fresh as the crate graph.
pub type DiskCache = FxHashMap<(AbsPathBuf, AbsPathBuf), bool>;

/// What is needed to compare the sources of a crate of a worktree with those of its base.
pub struct Sources<'a> {
    pub vfs: &'a Vfs,
    pub roots: &'a SourceRoots,
    pub pulled_in_files: &'a FxHashMap<FileId, Vec<PulledInFile>>,
    pub disk_cache: &'a RefCell<DiskCache>,
}

impl Sources<'_> {
    /// Whether the files at the two paths have the same contents, or neither exists, and so
    /// for the files that they pull in.
    fn same_file(
        &self,
        overlay: &Overlay,
        path: &AbsPath,
        base_path: &AbsPath,
        visited: &mut FxHashSet<FileId>,
    ) -> bool {
        let loaded = |path: &AbsPath| {
            let (file, _) = self.vfs.file_id(&VfsPath::from(path.to_path_buf()))?;
            Some((file, self.vfs.content_hash(file)?))
        };
        match (loaded(path), loaded(base_path)) {
            (Some((file, hash)), Some((_, base_hash))) => {
                hash == base_hash
                    && (!visited.insert(file)
                        || self.pulls_in_the_same(overlay, file, path, visited))
            }
            // Only one of them is loaded, for example because a client has it open. What we
            // remember about the files on disk says nothing about the loaded one.
            (Some((_, hash)), None) | (None, Some((_, hash))) => {
                let on_disk = if loaded(path).is_some() { base_path } else { path };
                fs::read(on_disk).is_ok_and(|contents| {
                    hash_once::<FxHasher>(&*contents) == hash
                        && (path.extension() != Some("rs")
                            || str::from_utf8(&contents).is_ok_and(|text| {
                                self.all_the_same(overlay, &pulled_in_files(text), path, visited)
                            }))
                })
            }
            // Not every file is loaded, for example the ones that are not Rust sources.
            (None, None) => {
                let key = (path.to_path_buf(), base_path.to_path_buf());
                if let Some(&same) = self.disk_cache.borrow().get(&key) {
                    return same;
                }
                // Guards against files that pull in each other.
                self.disk_cache.borrow_mut().insert(key.clone(), true);
                let same = match (fs::read(path), fs::read(base_path)) {
                    (Ok(contents), Ok(base_contents)) => {
                        contents == base_contents
                            && (path.extension() != Some("rs")
                                || str::from_utf8(&contents).is_ok_and(|text| {
                                    self.all_the_same(
                                        overlay,
                                        &pulled_in_files(text),
                                        path,
                                        visited,
                                    )
                                }))
                    }
                    (Err(_), Err(_)) => true,
                    _ => false,
                };
                self.disk_cache.borrow_mut().insert(key, same);
                same
            }
        }
    }

    /// Whether the files that the worktree's file at `path` pulls in, `pulled_in`, are the same
    /// as what its counterpart in the base checkout pulls in.
    fn all_the_same(
        &self,
        overlay: &Overlay,
        pulled_in: &[PulledInFile],
        path: &AbsPath,
        visited: &mut FxHashSet<FileId>,
    ) -> bool {
        pulled_in.iter().all(|pulled_in| match pulled_in {
            PulledInFile::Relative(relative) => {
                let Some(pulled_in) = path.parent().map(|dir| dir.absolutize(relative)) else {
                    return false;
                };
                // A file outside of the worktree is the same file for both, but then the two
                // crates are not at the same place relative to it, so they see other files.
                let Some(base_pulled_in) = overlay.to_base(&pulled_in) else {
                    return false;
                };
                self.same_file(overlay, &pulled_in, &base_pulled_in, visited)
            }
            PulledInFile::Unknown => false,
        })
    }

    /// Whether the files that the worktree's `file` at `path` pulls in by path are the same as
    /// what its counterpart in the base checkout pulls in.
    fn pulls_in_the_same(
        &self,
        overlay: &Overlay,
        file: FileId,
        path: &AbsPath,
        visited: &mut FxHashSet<FileId>,
    ) -> bool {
        match self.pulled_in_files.get(&file) {
            Some(pulled_in) => self.all_the_same(overlay, pulled_in, path, visited),
            None => true,
        }
    }
}

/// Whether the source root of `worktree_file` has the same files with the same contents as the
/// source root of `base_file`, and so do the files they pull in by path.
pub fn same_sources(
    sources: &Sources<'_>,
    overlay: &Overlay,
    worktree_file: FileId,
    base_file: FileId,
) -> bool {
    let Sources { vfs, roots, .. } = sources;
    let (Some(worktree_root), Some(base_root)) = (roots.of(worktree_file), roots.of(base_file))
    else {
        return false;
    };
    if worktree_root.iter().count() != base_root.iter().count() {
        return false;
    }
    worktree_root.iter().all(|file| {
        let Some(path) = worktree_root.path_for_file(&file).and_then(|path| path.as_path()) else {
            return false;
        };
        let base_file = overlay
            .to_base(path)
            .and_then(|path| base_root.file_for_path(&VfsPath::from(path)).copied());
        base_file.is_some_and(|base_file| vfs.content_hash(file) == vfs.content_hash(base_file))
            && sources.pulls_in_the_same(overlay, file, path, &mut FxHashSet::from_iter([file]))
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
    let (entries, other_entries) = match (entries(dir), entries(other)) {
        (Some(entries), Some(other_entries)) => (entries, other_entries),
        // Neither exists: nothing was generated that could differ.
        (None, None) => return !dir.exists() && !other.exists(),
        _ => return false,
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
pub fn base_crate(
    sources: &Sources<'_>,
    overlay: &Overlay,
    overlay_crates: &mut OverlayCrates,
    graph: &CrateGraphBuilder,
    krate: &CrateBuilder,
) -> Option<CrateBuilderId> {
    let vfs = sources.vfs;
    let worktree_file = krate.basic.root_file_id;
    let base_file = match overlay.to_base(vfs.file_path(worktree_file).as_path()?) {
        Some(base_path) => {
            let (base_file, _) = vfs.file_id(&VfsPath::from(base_path))?;
            let same_sources = same_sources(sources, overlay, worktree_file, base_file);
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
/// the same, as are the sources of their packages, and the crates of the package of `file` are
/// shared with the base checkout.
///
/// A package may have crates that are shared and crates that are not, for example a test that
/// depends on a crate that differs. With `all_crates`, such a package does not count.
pub fn shared_base_file(
    vfs: &Vfs,
    roots: &SourceRoots,
    overlays: &[Overlay],
    overlay_crates: &OverlayCrates,
    file: FileId,
    all_crates: bool,
) -> Option<FileId> {
    let path = vfs.file_path(file).as_path()?;
    let overlay = overlays.iter().find(|it| path.starts_with(&it.worktree_root))?;
    let (base_file, _) = vfs.file_id(&VfsPath::from(overlay.to_base(path)?))?;
    if vfs.content_hash(file) != vfs.content_hash(base_file) {
        return None;
    }
    let crates_of_package = || {
        overlay_crates
            .iter()
            .filter(|&(&(worktree_root_file, _), _)| roots.in_same_root(worktree_root_file, file))
            .map(|(_, krate)| krate)
    };
    let shared = crates_of_package().all(|krate| krate.same_sources)
        && crates_of_package().any(|krate| krate.shared)
        && (!all_crates || crates_of_package().all(|krate| krate.shared));
    shared.then_some(base_file)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayCrate {
    /// Whether the sources of the crate were the same as in the base checkout when the crate
    /// graph was built.
    pub same_sources: bool,
    /// Whether the crate of the base checkout stands in for the crate.
    pub shared: bool,
}

/// The crates of worktrees that have a counterpart in the base checkout, by the root files of
/// both.
pub type OverlayCrates = FxHashMap<(FileId, FileId), OverlayCrate>;

/// Builds the crate graph of `workspaces`. A crate of a workspace that is an overlay, as told
/// by `overlays`, is replaced by the crate of its base checkout that stands in for it, if
/// `sources` are given to compare the two with. What was decided is recorded in
/// `overlay_crates`.
pub fn crate_graph(
    workspaces: &[ProjectWorkspace],
    extra_env: &FxHashMap<String, Option<String>>,
    mut load: impl FnMut(&AbsPath) -> Option<FileId>,
    overlays: &[Option<Overlay>],
    sources: Option<&Sources<'_>>,
    overlay_crates: &mut OverlayCrates,
) -> (CrateGraphBuilder, Vec<ProcMacroPaths>) {
    let mut crate_graph = CrateGraphBuilder::default();
    let mut proc_macro_paths = vec![ProcMacroPaths::default(); workspaces.len()];
    let overlay_of = |idx: usize| overlays.get(idx).and_then(Option::as_ref);
    // The base checkouts have to be in the graph before the worktrees that are overlaid on them.
    let (overlaid, plain): (Vec<usize>, Vec<usize>) =
        (0..workspaces.len()).partition(|&idx| overlay_of(idx).is_some());
    for idx in plain.into_iter().chain(overlaid) {
        let (other, mut crate_proc_macros) = workspaces[idx].to_crate_graph(&mut load, extra_env);

        match (overlay_of(idx), sources) {
            (Some(overlay), Some(sources)) => {
                crate_graph.extend_with(other, &mut crate_proc_macros, |graph, krate| {
                    base_crate(sources, overlay, overlay_crates, graph, krate)
                })
            }
            _ => crate_graph.extend(other, &mut crate_proc_macros),
        };
        proc_macro_paths[idx] = crate_proc_macros;
    }

    crate_graph.shrink_to_fit();
    proc_macro_paths.shrink_to_fit();
    (crate_graph, proc_macro_paths)
}

#[cfg(test)]
mod tests {
    use super::{PulledInFile, pulled_in_files};

    #[test]
    fn finds_files_pulled_in_by_path() {
        let relative = |path: &str| vec![PulledInFile::Relative(path.to_owned())];
        assert_eq!(
            pulled_in_files("#[path = \"../shared.rs\"]\nmod shared;"),
            relative("../shared.rs")
        );
        assert_eq!(pulled_in_files("include!(\"../../gen.rs\");"), relative("../../gen.rs"));
        assert_eq!(pulled_in_files("#[path = \"imp/unix.rs\"]\nmod imp;"), relative("imp/unix.rs"));
        assert_eq!(
            pulled_in_files("const S: &str = include_str!(\"/etc/hosts\");"),
            vec![PulledInFile::Unknown]
        );
        assert_eq!(
            pulled_in_files("include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/x.rs\"));"),
            vec![PulledInFile::Unknown]
        );
        // What a build script generated is compared separately
        assert_eq!(pulled_in_files("include!(concat!(env!(\"OUT_DIR\"), \"/x.rs\"));"), vec![]);
        assert_eq!(
            pulled_in_files("#[cfg_attr(unix, path = \"unix.rs\")]\nmod imp;"),
            relative("unix.rs")
        );
        // Inside of an inline module the path is relative to another directory
        assert_eq!(
            pulled_in_files("mod inline {\n    #[path = \"../a.rs\"]\n    mod a;\n}"),
            vec![PulledInFile::Unknown]
        );
        assert_eq!(pulled_in_files("fn f() { include!(\"gen.rs\"); }"), relative("gen.rs"));
        // Prose and unrelated code do not count
        assert_eq!(pulled_in_files("// include the `../../shared.rs` file\nmod a;"), vec![]);
        assert_eq!(
            pulled_in_files("fn f(path: &str) { g(\"../x\") }\n/* include!(\"/a\") */"),
            vec![]
        );
        assert_eq!(pulled_in_files("let include = path.join(\"../x\");"), vec![]);
        assert_eq!(pulled_in_files("fn f() { format!(\"{path}\", path = \"../x\"); }"), vec![]);
    }
}
