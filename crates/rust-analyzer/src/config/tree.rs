use indextree::NodeId;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use std::{fmt, sync::Arc};
use vfs::{FileId, Vfs};

use super::{ConfigInput, LocalConfigData, RootLocalConfigData};

pub struct ConcurrentConfigTree {
    // One rwlock on the whole thing is probably fine.
    // If you have 40,000 crates and you need to edit your config 200x/second, let us know.
    rwlock: RwLock<ConfigTree>,
}

impl ConcurrentConfigTree {
    fn new(config_tree: ConfigTree) -> Self {
        Self { rwlock: RwLock::new(config_tree) }
    }
}

impl fmt::Debug for ConcurrentConfigTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rwlock.read().fmt(f)
    }
}

#[derive(Debug)]
pub enum ConfigTreeError {
    Removed,
    NonExistent,
    Utf8(vfs::VfsPath, std::str::Utf8Error),
    TomlParse(vfs::VfsPath, toml::de::Error),
    TomlDeserialize { path: vfs::VfsPath, field: String, error: toml::de::Error },
}

/// Some rust-analyzer.toml files have changed, and/or the LSP client sent a new configuration.
pub struct ConfigChanges {
    ra_toml_changes: Vec<vfs::ChangedFile>,
    /// - `None` => no change
    /// - `Some(None)` => the client config was removed / reset or something
    /// - `Some(Some(...))` => the client config was updated
    client_change: Option<Option<Arc<ConfigInput>>>,
    parent_changes: FxHashMap<FileId, ConfigParent>,
}

#[derive(Debug)]
pub struct ConfigParentChange {
    /// The config node in question
    pub file_id: FileId,
    pub parent: ConfigParent,
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
    /// let config_parent_changes = [
    ///   ConfigParentChange { node: root, parent: ConfigParent::UserDefault },
    ///   ConfigParentChange { node: crate_a, parent: ConfigParent::Parent(root) },
    ///   ConfigParentChange { node: crate_b, parent: ConfigParent::Parent(crate_a) }
    /// ];
    /// ```
    Parent(FileId),
}

impl ConcurrentConfigTree {
    pub fn apply_changes(&self, changes: ConfigChanges, vfs: &Vfs) -> Vec<ConfigTreeError> {
        let mut errors = Vec::new();
        self.rwlock.write().apply_changes(changes, vfs, &mut errors);
        errors
    }
    pub fn read_config(&self, file_id: FileId) -> Result<Arc<LocalConfigData>, ConfigTreeError> {
        let reader = self.rwlock.upgradable_read();
        if let Some(computed) = reader.read_only(file_id)? {
            return Ok(computed);
        } else {
            let mut writer = RwLockUpgradableReadGuard::upgrade(reader);
            return writer.compute(file_id);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ConfigSource {
    XdgConfig(FileId),
    RaToml(FileId),
}

slotmap::new_key_type! {
    struct ComputedIdx;
}

#[derive(Debug)]
struct ConfigNode {
    src: ConfigSource,
    input: Option<Arc<ConfigInput>>,
    computed_idx: ComputedIdx,
}

struct ConfigTree {
    tree: indextree::Arena<ConfigNode>,
    client_config: Option<Arc<ConfigInput>>,
    xdg_config_file_id: FileId,
    xdg_config_node_id: NodeId,
    ra_file_id_map: FxHashMap<FileId, NodeId>,
    computed: SlotMap<ComputedIdx, Option<Arc<LocalConfigData>>>,
    with_client_config: slotmap::SecondaryMap<ComputedIdx, Arc<LocalConfigData>>,
}

fn parse_toml(
    file_id: FileId,
    vfs: &Vfs,
    scratch: &mut Vec<(String, toml::de::Error)>,
    errors: &mut Vec<ConfigTreeError>,
) -> Option<Arc<ConfigInput>> {
    let content = vfs.file_contents(file_id);
    let path = vfs.file_path(file_id);
    if content.is_empty() {
        return None;
    }
    let content_str = match std::str::from_utf8(content) {
        Err(e) => {
            tracing::error!("non-UTF8 TOML content for {path}: {e}");
            errors.push(ConfigTreeError::Utf8(path, e));
            return None;
        }
        Ok(str) => str,
    };
    let table = match toml::from_str(content_str) {
        Ok(table) => table,
        Err(e) => {
            errors.push(ConfigTreeError::TomlParse(path, e));
            return None;
        }
    };
    let input = Arc::new(ConfigInput::from_toml(table, scratch));
    scratch.drain(..).for_each(|(field, error)| {
        errors.push(ConfigTreeError::TomlDeserialize { path: path.clone(), field, error });
    });
    Some(input)
}

impl ConfigTree {
    fn new(xdg_config_file_id: FileId) -> Self {
        let mut tree = indextree::Arena::new();
        let mut computed = SlotMap::default();
        let mut ra_file_id_map = FxHashMap::default();
        let xdg_config_node_id = tree.new_node(ConfigNode {
            src: ConfigSource::XdgConfig(xdg_config_file_id),
            input: None,
            computed_idx: computed.insert(Option::<Arc<LocalConfigData>>::None),
        });
        ra_file_id_map.insert(xdg_config_file_id, xdg_config_node_id);

        Self {
            client_config: None,
            xdg_config_file_id,
            xdg_config_node_id,
            ra_file_id_map,
            tree,
            computed,
            with_client_config: Default::default(),
        }
    }

    fn read_only(&self, file_id: FileId) -> Result<Option<Arc<LocalConfigData>>, ConfigTreeError> {
        let node_id = *self.ra_file_id_map.get(&file_id).ok_or(ConfigTreeError::NonExistent)?;
        self.read_only_inner(node_id)
    }

    fn read_only_inner(
        &self,
        node_id: NodeId,
    ) -> Result<Option<Arc<LocalConfigData>>, ConfigTreeError> {
        // indextree does not check this during get(), probably for perf reasons?
        // get() is apparently only a bounds check
        if node_id.is_removed(&self.tree) {
            return Err(ConfigTreeError::Removed);
        }
        let node = self.tree.get(node_id).ok_or(ConfigTreeError::NonExistent)?.get();
        let stored = self.with_client_config.get(node.computed_idx).cloned();
        tracing::trace!(
            "read_only_inner on {:?} got {:?}",
            node.src,
            stored.as_ref().map(|_| "some stored value")
        );
        Ok(stored)
    }

    fn compute(&mut self, file_id: FileId) -> Result<Arc<LocalConfigData>, ConfigTreeError> {
        let node_id = *self.ra_file_id_map.get(&file_id).ok_or(ConfigTreeError::NonExistent)?;
        let (computed, idx) = self.compute_recursive(node_id)?;
        let out = if let Some(client_config) = self.client_config.as_deref() {
            Arc::new(computed.clone_with_overrides(client_config.local.clone()))
        } else {
            computed
        };
        self.with_client_config.insert(idx, out.clone());
        Ok(out)
    }
    fn compute_recursive(
        &mut self,
        node_id: NodeId,
    ) -> Result<(Arc<LocalConfigData>, ComputedIdx), ConfigTreeError> {
        if node_id.is_removed(&self.tree) {
            return Err(ConfigTreeError::Removed);
        }
        let node = self.tree.get(node_id).ok_or(ConfigTreeError::NonExistent)?.get();
        tracing::trace!("compute_inner on {:?}", node.src);
        let idx = node.computed_idx;
        let slot = &mut self.computed[idx];
        if let Some(slot) = slot {
            Ok((slot.clone(), idx))
        } else {
            let self_computed = if let Some(parent) =
                self.tree.get(node_id).ok_or(ConfigTreeError::NonExistent)?.parent()
            {
                tracing::trace!("looking at parent of {node_id:?} -> {parent:?}");
                let self_input = node.input.clone();
                let (parent_computed, _) = self.compute_recursive(parent)?;
                if let Some(input) = self_input.as_deref() {
                    Arc::new(parent_computed.clone_with_overrides(input.local.clone()))
                } else {
                    parent_computed
                }
            } else {
                tracing::trace!("{node_id:?} is a root node");
                // We have hit a root node
                let self_input = node.input.clone();
                if let Some(input) = self_input.as_deref() {
                    let root_local = RootLocalConfigData::from_root_input(input.local.clone());
                    Arc::new(root_local.0)
                } else {
                    Arc::new(LocalConfigData::default())
                }
            };
            // Get a new &mut slot because self.compute(parent) also gets mut access
            let slot = &mut self.computed[idx];
            slot.replace(self_computed.clone());
            Ok((self_computed, idx))
        }
    }

    fn insert_toml(&mut self, file_id: FileId, input: Option<Arc<ConfigInput>>) -> NodeId {
        let computed = self.computed.insert(None);
        let node_id = self.tree.new_node(ConfigNode {
            src: ConfigSource::RaToml(file_id),
            input,
            computed_idx: computed,
        });
        if let Some(_removed) = self.ra_file_id_map.insert(file_id, node_id) {
            panic!("ERROR: node should not have existed for {file_id:?} but it did");
        }
        // By default, everything is under the xdg_config_node_id
        self.xdg_config_node_id.append(node_id, &mut self.tree);
        node_id
    }

    fn update_toml(
        &mut self,
        file_id: FileId,
        input: Option<Arc<ConfigInput>>,
    ) -> Result<NodeId, ConfigTreeError> {
        let Some(node_id) = self.ra_file_id_map.get(&file_id).cloned() else {
            let node_id = self.insert_toml(file_id, input);
            return Ok(node_id);
        };
        if node_id.is_removed(&self.tree) {
            return Err(ConfigTreeError::Removed);
        }
        let node = self.tree.get_mut(node_id).ok_or(ConfigTreeError::NonExistent)?;
        node.get_mut().input = input;

        self.invalidate_subtree(node_id);
        // tracing::trace!("invalidated subtree:\n{:#?}", node_id.debug_pretty_print(&self.tree));
        Ok(node_id)
    }

    fn ensure_node(&mut self, file_id: FileId) -> NodeId {
        let Some(&node_id) = self.ra_file_id_map.get(&file_id) else {
            return self.insert_toml(file_id, None);
        };
        node_id
    }

    fn invalidate_subtree(&mut self, node_id: NodeId) {
        //
        // This is why we need the computed values outside the indextree: we iterate immutably
        // over the tree while holding a &mut self.computed.
        node_id.descendants(&self.tree).for_each(|x| {
            let Some(desc) = self.tree.get(x) else {
                return;
            };
            let desc = desc.get();
            let Some(slot) = self.computed.get_mut(desc.computed_idx) else {
                tracing::error!(
                    "computed_idx missing from computed local config slotmap: {:?}",
                    desc.computed_idx
                );
                return;
            };
            tracing::trace!("invalidating {x:?} / {:?}", desc.src);
            slot.take();

            // Also invalidate the secondary data
            self.with_client_config.remove(desc.computed_idx);
        });
    }

    fn remove_toml(&mut self, file_id: FileId) -> Option<()> {
        let node_id = *self.ra_file_id_map.get(&file_id)?;
        if node_id.is_removed(&self.tree) {
            return None;
        }
        let node = self.tree.get_mut(node_id)?.get_mut();
        node.input = None;
        self.invalidate_subtree(node_id);
        Some(())
    }

    fn apply_changes(
        &mut self,
        changes: ConfigChanges,
        vfs: &Vfs,
        errors: &mut Vec<ConfigTreeError>,
    ) {
        let mut scratch_errors = Vec::new();
        let ConfigChanges { client_change, ra_toml_changes, mut parent_changes } = changes;

        if let Some(change) = client_change {
            match (self.client_config.as_ref(), change.as_ref()) {
                (None, None) => {}
                (Some(a), Some(b)) if Arc::ptr_eq(a, b) => {}
                _ => {
                    // invalidate the output table only, don't immediately need to recompute
                    // everything from scratch
                    self.with_client_config.clear();
                    self.client_config = change;
                }
            }
        }

        for change in ra_toml_changes {
            // turn and face the strain
            match change.change_kind {
                vfs::ChangeKind::Create | vfs::ChangeKind::Modify => {
                    if change.change_kind == vfs::ChangeKind::Create {
                        parent_changes.entry(change.file_id).or_insert(ConfigParent::UserDefault);
                    }
                    let input = parse_toml(change.file_id, vfs, &mut scratch_errors, errors);
                    tracing::trace!("updating toml for {:?} to {:?}", change.file_id, input);

                    if let Err(e) = self.update_toml(change.file_id, input) {
                        errors.push(e);
                    }
                }
                vfs::ChangeKind::Delete => {
                    self.remove_toml(change.file_id);
                }
            }
        }

        for (file_id, parent) in parent_changes {
            let node_id = self.ensure_node(file_id);
            let parent_node_id = match parent {
                ConfigParent::Parent(parent_file_id) => self.ensure_node(parent_file_id),
                ConfigParent::UserDefault if file_id == self.xdg_config_file_id => continue,
                ConfigParent::UserDefault => self.xdg_config_node_id,
            };
            // order of children within the parent node does not matter
            tracing::trace!("appending child {node_id:?} to {parent_node_id:?}");
            parent_node_id.append(node_id, &mut self.tree);
        }
    }
}

impl fmt::Debug for ConfigTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vfs::{AbsPathBuf, VfsPath};

    fn alloc_file_id(vfs: &mut Vfs, s: &str) -> FileId {
        tracing_subscriber::fmt().init();
        let abs_path = AbsPathBuf::try_from(PathBuf::new().join(s)).unwrap();

        let vfs_path = VfsPath::from(abs_path);
        // FIXME: the vfs should expose this functionality more simply.
        // We shouldn't have to clone the vfs path just to get a FileId.
        let file_id = vfs.alloc_file_id(vfs_path);
        vfs.set_file_id_contents(file_id, None);
        file_id
    }

    fn alloc_config(vfs: &mut Vfs, s: &str, config: &str) -> FileId {
        let abs_path = AbsPathBuf::try_from(PathBuf::new().join(s)).unwrap();

        let vfs_path = VfsPath::from(abs_path);
        // FIXME: the vfs should expose this functionality more simply.
        // We shouldn't have to clone the vfs path just to get a FileId.
        let file_id = vfs.alloc_file_id(vfs_path);
        vfs.set_file_id_contents(file_id, Some(config.to_string().into_bytes()));
        file_id
    }

    use super::*;
    #[test]
    fn basic() {
        let mut vfs = Vfs::default();
        let xdg_config_file_id =
            alloc_file_id(&mut vfs, "/home/username/.config/rust-analyzer/rust-analyzer.toml");
        let config_tree = ConcurrentConfigTree::new(ConfigTree::new(xdg_config_file_id));

        let root = alloc_config(
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

        let mut parent_changes = FxHashMap::default();
        parent_changes.insert(crate_a, ConfigParent::Parent(root));

        let changes = ConfigChanges {
            // Normally you will filter these!
            ra_toml_changes: vfs.take_changes(),
            parent_changes,
            client_change: Some(Some(Arc::new(ConfigInput {
                local: crate::config::LocalConfigInput {
                    semanticHighlighting_strings_enable: Some(false),
                    ..Default::default()
                },
                ..Default::default()
            }))),
        };

        dbg!(config_tree.apply_changes(changes, &vfs));

        let local = config_tree.read_config(crate_a).unwrap();
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
        "#
                .to_string()
                .into_bytes(),
            ),
        );

        let changes = ConfigChanges {
            ra_toml_changes: dbg!(vfs.take_changes()),
            parent_changes: Default::default(),
            client_change: None,
        };
        dbg!(config_tree.apply_changes(changes, &vfs));

        let prev = local;
        let local = config_tree.read_config(crate_a).unwrap();
        assert!(!Arc::ptr_eq(&prev, &local));
        let local2 = config_tree.read_config(crate_a).unwrap();
        assert!(Arc::ptr_eq(&local, &local2));

        assert_eq!(
            local.inlayHints_discriminantHints_enable,
            crate::config::DiscriminantHintsDef::Always
        );
    }
}
