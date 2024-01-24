use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;
use vfs::{AbsPathBuf, FileId, Vfs};

use super::{ConfigInput, LocalConfigData, RootLocalConfigData};

#[derive(Debug)]
pub enum ConfigTreeError {
    Removed,
    NonExistent,
    Utf8(FileId, std::str::Utf8Error),
    TomlParse(FileId, toml::de::Error),
    TomlDeserialize { file_id: FileId, field: String, error: toml::de::Error },
}

/// Some rust-analyzer.toml files have changed, and/or the LSP client sent a new configuration.
pub struct ConfigChanges {
    /// - `None` => no change
    /// - `Some(None)` => the client config was removed / reset or something
    /// - `Some(Some(...))` => the client config was updated
    client_change: Option<Option<Arc<ConfigInput>>>,
    set_project_root: Option<AbsPathBuf>,
    set_source_roots: Option<FxHashSet<AbsPathBuf>>,
    ra_toml_changes: Vec<vfs::ChangedFile>,
}

/// Internal version
struct ConfigChangesInner {
    /// - `None` => no change
    /// - `Some(None)` => the client config was removed / reset or something
    /// - `Some(Some(...))` => the client config was updated
    client_change: Option<Option<Arc<ConfigInput>>>,
    parent_changes: FxHashMap<FileId, ConfigParent>,
    ra_toml_changes: Vec<vfs::ChangedFile>,
}

#[derive(Debug)]
pub enum ConfigParent {
    /// The node is now a root in its own right, but still inherits from the config in XDG_CONFIG_HOME
    /// etc
    UserDefault,
    /// The node is now a child of another rust-analyzer.toml. Even if that one is a non-existent
    /// file, it's fine.
    ///
    ///
    /// ```ignore,text
    /// /project_root/
    ///   rust-analyzer.toml
    ///   crate_a/
    ///      crate_b/
    ///        rust-analyzer.toml
    ///
    /// ```
    ///
    /// ```ignore
    /// // imagine set_file_contents = vfs.set_file_contents() and then get the vfs.file_id()
    ///
    /// let root = vfs.set_file_contents("/project_root/rust-analyzer.toml", Some("..."));
    /// let crate_a = vfs.set_file_contents("/project_root/crate_a/rust-analyzer.toml", None);
    /// let crate_b = vfs.set_file_contents("/project_root/crate_a/crate_b/rust-analyzer.toml", Some("..."));
    /// let parent_changes = FxHashMap::from_iter([
    ///   (root, ConfigParent::UserDefault),
    ///   (crate_a, ConfigParent::Parent(root)),
    ///   (crate_b, ConfigParent::Parent(crate_a)),
    /// ]);
    /// ```
    Parent(FileId),
}

/// Easier and probably more performant than making ConfigInput implement Eq
#[derive(Debug)]
struct PointerCmp<T>(Arc<T>);
impl<T> PointerCmp<T> {
    fn new(t: T) -> Self {
        Self(Arc::new(t))
    }
}
impl<T> Clone for PointerCmp<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> PartialEq for PointerCmp<T> {
    fn eq(&self, other: &Self) -> bool {
        (Arc::as_ptr(&self.0) as *const ()).eq(&Arc::as_ptr(&other.0).cast())
    }
}
impl<T> Eq for PointerCmp<T> {}
impl<T> std::ops::Deref for PointerCmp<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

#[salsa::query_group(ConfigTreeStorage)]
trait ConfigTreeQueries {
    #[salsa::input]
    fn client_config(&self) -> Option<PointerCmp<ConfigInput>>;

    #[salsa::input]
    fn parent(&self, file_id: FileId) -> Option<FileId>;

    #[salsa::input]
    fn config_input(&self, file_id: FileId) -> Option<PointerCmp<ConfigInput>>;

    fn recursive_local(&self, file_id: FileId) -> PointerCmp<LocalConfigData>;

    /// The output
    fn computed_local_config(&self, file_id: FileId) -> PointerCmp<LocalConfigData>;
}

fn recursive_local(db: &dyn ConfigTreeQueries, file_id: FileId) -> PointerCmp<LocalConfigData> {
    let self_input = db.config_input(file_id);
    tracing::trace!(?self_input, ?file_id);
    match db.parent(file_id) {
        Some(parent) if parent != file_id => {
            let parent_computed = db.recursive_local(parent);
            if let Some(input) = self_input.as_deref() {
                PointerCmp::new(parent_computed.clone_with_overrides(input.local.clone()))
            } else {
                parent_computed
            }
        }
        _ => {
            // this is a root node, or we just broke a cycle
            if let Some(input) = self_input.as_deref() {
                let root_local = RootLocalConfigData::from_root_input(input.local.clone());
                PointerCmp::new(root_local.0)
            } else {
                PointerCmp::new(LocalConfigData::default())
            }
        }
    }
}

fn computed_local_config(
    db: &dyn ConfigTreeQueries,
    file_id: FileId,
) -> PointerCmp<LocalConfigData> {
    let computed = db.recursive_local(file_id);
    if let Some(client) = db.client_config() {
        PointerCmp::new(computed.clone_with_overrides(client.local.clone()))
    } else {
        computed
    }
}

#[salsa::database(ConfigTreeStorage)]
pub struct ConfigDb {
    storage: salsa::Storage<Self>,
    known_file_ids: FxHashSet<FileId>,
    xdg_config_file_id: FileId,
    source_roots: FxHashSet<AbsPathBuf>,
    project_root: AbsPathBuf,
}

impl salsa::Database for ConfigDb {}

impl ConfigDb {
    pub fn new(xdg_config_file_id: FileId, project_root: AbsPathBuf) -> Self {
        let mut this = Self {
            storage: Default::default(),
            known_file_ids: FxHashSet::default(),
            xdg_config_file_id,
            source_roots: FxHashSet::default(),
            project_root,
        };
        this.set_client_config(None);
        this.ensure_node(xdg_config_file_id);
        this
    }

    /// Gets the value of LocalConfigData for a given `rust-analyzer.toml` FileId.
    ///
    /// The rust-analyzer.toml does not need to exist on disk. All values are the expression of
    /// overriding the parent `rust-analyzer.toml`, set by adding an entry in
    /// `ConfigChanges.parent_changes`.
    ///
    /// If the db is not aware of the given `rust-analyzer.toml` FileId, then the config is read
    /// from the user's system-wide default config.
    ///
    /// Note that the client config overrides all configs.
    pub fn local_config(&self, ra_toml_file_id: FileId) -> Arc<LocalConfigData> {
        if self.known_file_ids.contains(&ra_toml_file_id) {
            self.computed_local_config(ra_toml_file_id).0
        } else {
            tracing::warn!(?ra_toml_file_id, "called local_config with unknown file id");
            self.computed_local_config(self.xdg_config_file_id).0
        }
    }

    /// Applies a bunch of [`ConfigChanges`]. The FileIds referred to in `ConfigChanges` do not
    /// need to exist.
    pub fn apply_changes(&mut self, changes: ConfigChanges, vfs: &mut Vfs) -> Vec<ConfigTreeError> {
        if let Some(new_project_root) = &changes.set_project_root {
            self.project_root = new_project_root.clone();
        }
        let source_root_change = changes.set_source_roots.as_ref().or_else(|| {
            if changes.set_project_root.is_some() {
                Some(&self.source_roots)
            } else {
                None
            }
        });
        let parent_changes = if let Some(source_roots) = source_root_change {
            source_roots
                .iter()
                .flat_map(|path: &AbsPathBuf| {
                    path.ancestors()
                        .take_while(|x| x.starts_with(&self.project_root))
                        .map(|dir| dir.join("rust-analyzer.toml"))
                        .map(|path| vfs.alloc_file_id(path.into()))
                        .collect_vec()
                        // immediately get tuple_windows before returning from flat_map
                        .into_iter()
                        .tuple_windows()
                        .map(|(a, b)| (a, ConfigParent::Parent(b)))
                })
                .collect::<FxHashMap<_, _>>()
        } else {
            Default::default()
        };

        if tracing::enabled!(tracing::Level::TRACE) {
            for (&a, parent) in &parent_changes {
                tracing::trace!(
                    "{a:?} ({:?}): parent = {parent:?} ({:?})",
                    vfs.file_path(a),
                    match parent {
                        ConfigParent::Parent(p) => vfs.file_path(*p).to_string(),
                        ConfigParent::UserDefault => "xdg".to_string(),
                    }
                );
            }
        }

        // Could delete (self.known_file_ids - parent_changes.keys) here.

        let inner = ConfigChangesInner {
            ra_toml_changes: changes.ra_toml_changes,
            client_change: changes.client_change,
            parent_changes,
        };
        self.apply_changes_inner(inner, vfs)
    }

    fn apply_changes_inner(
        &mut self,
        changes: ConfigChangesInner,
        vfs: &Vfs,
    ) -> Vec<ConfigTreeError> {
        let mut scratch_errors = Vec::new();
        let mut errors = Vec::new();
        let ConfigChangesInner { client_change, ra_toml_changes, parent_changes } = changes;

        if let Some(change) = client_change {
            let current = self.client_config();
            let change = change.map(PointerCmp);
            match (current.as_ref(), change.as_ref()) {
                (None, None) => {}
                (Some(a), Some(b)) if a == b => {}
                _ => {
                    self.set_client_config(change);
                }
            }
        }

        for change in ra_toml_changes {
            // turn and face the strain
            match change.change_kind {
                vfs::ChangeKind::Create | vfs::ChangeKind::Modify => {
                    let input = parse_toml(change.file_id, vfs, &mut scratch_errors, &mut errors)
                        .map(PointerCmp);
                    tracing::trace!("updating toml for {:?} to {:?}", change.file_id, input);

                    self.ensure_node(change.file_id);
                    self.set_config_input(change.file_id, input);
                }
                vfs::ChangeKind::Delete => {
                    self.ensure_node(change.file_id);
                    self.set_config_input(change.file_id, None);
                }
            }
        }

        for (file_id, parent) in parent_changes {
            self.ensure_node(file_id);
            let parent_node_id = match parent {
                ConfigParent::Parent(parent_file_id) => {
                    self.ensure_node(parent_file_id);
                    parent_file_id
                }
                ConfigParent::UserDefault if file_id == self.xdg_config_file_id => continue,
                ConfigParent::UserDefault => self.xdg_config_file_id,
            };
            // order of children within the parent node does not matter
            tracing::trace!("appending child {file_id:?} to {parent_node_id:?}");
            self.set_parent(file_id, Some(parent_node_id))
        }

        errors
    }

    /// Inserts default values into the salsa inputs for the given file_id
    /// if it's never been seen before
    fn ensure_node(&mut self, file_id: FileId) {
        if self.known_file_ids.insert(file_id) {
            self.set_config_input(file_id, None);
            self.set_parent(
                file_id,
                if file_id == self.xdg_config_file_id {
                    None
                } else {
                    Some(self.xdg_config_file_id)
                },
            );
        }
    }
}

fn parse_toml(
    file_id: FileId,
    vfs: &Vfs,
    scratch: &mut Vec<(String, toml::de::Error)>,
    errors: &mut Vec<ConfigTreeError>,
) -> Option<Arc<ConfigInput>> {
    let content = vfs.file_contents(file_id);
    let content_str = match std::str::from_utf8(content) {
        Err(e) => {
            tracing::error!("non-UTF8 TOML content for {file_id:?}: {e}");
            errors.push(ConfigTreeError::Utf8(file_id, e));
            return None;
        }
        Ok(str) => str,
    };
    if content_str.is_empty() {
        return None;
    }
    let table = match toml::from_str(content_str) {
        Ok(table) => table,
        Err(e) => {
            errors.push(ConfigTreeError::TomlParse(file_id, e));
            return None;
        }
    };
    let input = Arc::new(ConfigInput::from_toml(table, scratch));
    scratch.drain(..).for_each(|(field, error)| {
        errors.push(ConfigTreeError::TomlDeserialize { file_id, field, error });
    });
    Some(input)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use itertools::Itertools;
    use vfs::{AbsPath, AbsPathBuf, VfsPath};

    fn alloc_file_id(vfs: &mut Vfs, s: &str) -> FileId {
        let abs_path = AbsPathBuf::try_from(PathBuf::new().join(s)).unwrap();

        let vfs_path = VfsPath::from(abs_path);
        let file_id = vfs.alloc_file_id(vfs_path);
        vfs.set_file_id_contents(file_id, None);
        file_id
    }

    fn alloc_config(vfs: &mut Vfs, s: &str, config: &str) -> FileId {
        let abs_path = AbsPathBuf::try_from(PathBuf::new().join(s)).unwrap();

        let vfs_path = VfsPath::from(abs_path);
        let file_id = vfs.alloc_file_id(vfs_path);
        vfs.set_file_id_contents(file_id, Some(config.to_string().into_bytes()));
        file_id
    }

    use super::*;
    #[test]
    fn basic() {
        tracing_subscriber::fmt().try_init().ok();
        let mut vfs = Vfs::default();
        let project_root = AbsPath::assert(Path::new("/root"));
        let xdg_config_file_id =
            alloc_file_id(&mut vfs, "/home/username/.config/rust-analyzer/rust-analyzer.toml");
        let mut config_tree = ConfigDb::new(xdg_config_file_id, project_root.to_path_buf());

        let source_roots = ["/root/crate_a"].map(Path::new).map(AbsPath::assert);

        let _root = alloc_config(
            &mut vfs,
            "/root/rust-analyzer.toml",
            r#"
            [completion.autoself]
            enable = false
            "#,
        );

        let crate_a = alloc_config(
            &mut vfs,
            "/root/crate_a/rust-analyzer.toml",
            r#"
            [completion.autoimport]
            enable = false
            # will be overridden by client
            [semanticHighlighting.strings]
            enable = true
            "#,
        );

        let new_source_roots = source_roots.into_iter().map(|abs| abs.to_path_buf()).collect();
        let changes = ConfigChanges {
            // Normally you will filter these!
            ra_toml_changes: vfs.take_changes(),
            set_project_root: None,
            set_source_roots: Some(new_source_roots),
            client_change: Some(Some(Arc::new(ConfigInput {
                local: crate::config::LocalConfigInput {
                    semanticHighlighting_strings_enable: Some(false),
                    ..Default::default()
                },
                ..Default::default()
            }))),
        };

        dbg!(config_tree.apply_changes(changes, &mut vfs));

        let local = config_tree.local_config(crate_a);
        // from root
        assert_eq!(local.completion_autoself_enable, false);
        // from crate_a
        assert_eq!(local.completion_autoimport_enable, false);
        // from client
        assert_eq!(local.semanticHighlighting_strings_enable, false);

        // --------------------------------------------------------

        // Now let's modify the xdg_config_file_id, which should invalidate everything else
        vfs.set_file_id_contents(
            xdg_config_file_id,
            Some(
                r#"
        # default is "never"
        [inlayHints.discriminantHints]
        enable = "always"
        [completion.autoself]
        enable = true
        [completion.autoimport]
        enable = true
        [semanticHighlighting.strings]
        enable = true
        "#
                .to_string()
                .into_bytes(),
            ),
        );

        let changes = ConfigChanges {
            client_change: None,
            set_project_root: None,
            set_source_roots: None,
            ra_toml_changes: dbg!(vfs.take_changes()),
        };
        dbg!(config_tree.apply_changes(changes, &mut vfs));

        let prev = local;
        let local = config_tree.local_config(crate_a);
        // Should have been recomputed
        assert!(!Arc::ptr_eq(&prev, &local));
        // But without changes in between, should give the same Arc back
        assert!(Arc::ptr_eq(&local, &config_tree.local_config(crate_a)));

        // The newly added xdg_config_file_id should affect the output if nothing else touches
        // this key
        assert_eq!(
            local.inlayHints_discriminantHints_enable,
            crate::config::DiscriminantHintsDef::Always
        );
        // But it should not win
        assert_eq!(local.completion_autoself_enable, false);
        assert_eq!(local.completion_autoimport_enable, false);
        assert_eq!(local.semanticHighlighting_strings_enable, false);
    }

    #[test]
    fn set_source_roots() {
        tracing_subscriber::fmt().try_init().ok();
        let mut vfs = Vfs::default();

        let project_root = AbsPath::assert(Path::new("/root"));
        let xdg =
            alloc_file_id(&mut vfs, "/home/username/.config/rust-analyzer/rust-analyzer.toml");
        let mut config_tree = ConfigDb::new(xdg, project_root.to_path_buf());

        let source_roots =
            ["/root/crate_a", "/root/crate_a/crate_b"].map(Path::new).map(AbsPath::assert);
        let source_root_tomls = source_roots
            .iter()
            .map(|dir| dir.join("rust-analyzer.toml"))
            .map(|path| AbsPathBuf::try_from(path).unwrap())
            .map(|path| vfs.alloc_file_id(path.into()))
            .collect_vec();
        let &[crate_a, crate_b] = &source_root_tomls[..] else {
            panic!();
        };

        vfs.set_file_id_contents(
            xdg,
            Some(b"[inlayHints.discriminantHints]\nenable = \"always\"".to_vec()),
        );
        vfs.set_file_id_contents(crate_a, Some(b"[completion.autoself]\nenable = false".to_vec()));
        // note that crate_b's rust-analyzer.toml doesn't exist

        let new_source_roots = source_roots.into_iter().map(|abs| abs.to_path_buf()).collect();
        let changes = ConfigChanges {
            client_change: None,
            set_project_root: None, // already set in ConfigDb::new(...)
            set_source_roots: Some(new_source_roots),
            ra_toml_changes: dbg!(vfs.take_changes()),
        };

        dbg!(config_tree.apply_changes(changes, &mut vfs));
        let local = config_tree.local_config(crate_b);

        assert_eq!(
            local.inlayHints_discriminantHints_enable,
            crate::config::DiscriminantHintsDef::Always
        );
        assert_eq!(local.completion_autoself_enable, false);
    }
}
