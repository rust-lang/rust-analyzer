use indextree::NodeId;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use std::sync::Arc;
use vfs::{FileId, Vfs};

use super::{ConfigInput, LocalConfigData, RootLocalConfigData};

pub struct ConcurrentConfigTree {
    // One rwlock on the whole thing is probably fine.
    // If you have 40,000 crates and you need to edit your config 200x/second, let us know.
    rwlock: RwLock<ConfigTree>,
}

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
    xdg_config_change: Option<Arc<ConfigInput>>,
    client_change: Option<Arc<ConfigInput>>,
    parent_changes: Vec<ConfigParentChange>,
}

pub struct ConfigParentChange {
    /// The config node in question
    pub node: FileId,
    pub parent: ConfigParent,
}

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
    ClientConfig,
    RaToml(FileId),
}

slotmap::new_key_type! {
    struct ComputedIdx;
}

struct ConfigNode {
    src: ConfigSource,
    // TODO: make option
    input: Arc<ConfigInput>,
    computed: ComputedIdx,
}

struct ConfigTree {
    tree: indextree::Arena<ConfigNode>,
    client_config: Arc<ConfigInput>,
    xdg_config_node_id: NodeId,
    ra_file_id_map: FxHashMap<FileId, NodeId>,
    computed: SlotMap<ComputedIdx, Option<Arc<LocalConfigData>>>,
}

fn parse_toml(
    file_id: FileId,
    vfs: &Vfs,
    scratch: &mut Vec<(String, toml::de::Error)>,
    errors: &mut Vec<ConfigTreeError>,
) -> Option<Arc<ConfigInput>> {
    let content = vfs.file_contents(file_id);
    let path = vfs.file_path(file_id);
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
        let xdg_config = tree.new_node(ConfigNode {
            src: ConfigSource::RaToml(xdg_config_file_id),
            input: Arc::new(ConfigInput::default()),
            computed: computed.insert(Option::<Arc<LocalConfigData>>::None),
        });
        ra_file_id_map.insert(xdg_config_file_id, xdg_config);

        Self {
            client_config: Arc::new(Default::default()),
            xdg_config_node_id: xdg_config,
            ra_file_id_map,
            tree,
            computed,
        }
    }

    fn read_only(&self, file_id: FileId) -> Result<Option<Arc<LocalConfigData>>, ConfigTreeError> {
        let node_id = *self.ra_file_id_map.get(&file_id).ok_or(ConfigTreeError::NonExistent)?;
        // indextree does not check this during get(), probably for perf reasons?
        // get() is apparently only a bounds check
        if node_id.is_removed(&self.tree) {
            return Err(ConfigTreeError::Removed);
        }
        let node = self.tree.get(node_id).ok_or(ConfigTreeError::NonExistent)?.get();
        Ok(self.computed[node.computed].clone())
    }

    fn compute(&mut self, file_id: FileId) -> Result<Arc<LocalConfigData>, ConfigTreeError> {
        let node_id = *self.ra_file_id_map.get(&file_id).ok_or(ConfigTreeError::NonExistent)?;
        self.compute_inner(node_id)
    }
    fn compute_inner(&mut self, node_id: NodeId) -> Result<Arc<LocalConfigData>, ConfigTreeError> {
        if node_id.is_removed(&self.tree) {
            return Err(ConfigTreeError::Removed);
        }
        let node = self.tree.get(node_id).ok_or(ConfigTreeError::NonExistent)?.get();
        let idx = node.computed;
        let slot = &mut self.computed[idx];
        if let Some(slot) = slot {
            Ok(slot.clone())
        } else {
            let self_computed = if let Some(parent) =
                self.tree.get(node_id).ok_or(ConfigTreeError::NonExistent)?.parent()
            {
                let self_input = node.input.clone();
                let parent_computed = self.compute_inner(parent)?;
                Arc::new(parent_computed.clone_with_overrides(self_input.local.clone()))
            } else {
                // We have hit a root node
                let self_input = node.input.clone();
                let root_local = RootLocalConfigData::from_root_input(self_input.local.clone());
                Arc::new(root_local.0)
            };
            // Get a new &mut slot because self.compute(parent) also gets mut access
            let slot = &mut self.computed[idx];
            slot.replace(self_computed.clone());
            Ok(self_computed)
        }
    }

    fn insert_toml(&mut self, file_id: FileId, input: Arc<ConfigInput>) -> NodeId {
        let computed = self.computed.insert(None);
        let node =
            self.tree.new_node(ConfigNode { src: ConfigSource::RaToml(file_id), input, computed });
        self.ra_file_id_map.insert(file_id, node);
        node
    }

    fn update_toml(
        &mut self,
        file_id: FileId,
        input: Arc<ConfigInput>,
    ) -> Result<(), ConfigTreeError> {
        let Some(node_id) = self.ra_file_id_map.get(&file_id).cloned() else {
            return Err(ConfigTreeError::NonExistent);
        };
        if node_id.is_removed(&self.tree) {
            return Err(ConfigTreeError::Removed);
        }
        let node = self.tree.get_mut(node_id).ok_or(ConfigTreeError::NonExistent)?;
        node.get_mut().input = input;

        self.invalidate_subtree(node_id);
        Ok(())
    }

    fn invalidate_subtree(&mut self, node_id: NodeId) {
        //
        // This is why we need the computed values outside the indextree: we iterate immutably
        // over the tree while holding a &mut self.computed.
        node_id.descendants(&self.tree).for_each(|x| {
            let Some(desc) = self.tree.get(x) else {
                return;
            };
            self.computed.get_mut(desc.get().computed).take();
        });
    }

    fn remove_toml(&mut self, file_id: FileId) -> Option<()> {
        let node_id = self.ra_file_id_map.remove(&file_id)?;
        if node_id.is_removed(&self.tree) {
            return None;
        }
        let node = self.tree.get(node_id)?;
        let idx = node.get().computed;
        let _ = self.computed.remove(idx);
        self.invalidate_subtree(node_id);
        Some(())
    }

    fn apply_changes(
        &mut self,
        mut changes: ConfigChanges,
        vfs: &Vfs,
        errors: &mut Vec<ConfigTreeError>,
    ) {
        let mut scratch_errors = Vec::new();
        let ConfigChanges { client_change, ra_toml_changes, xdg_config_change, parent_changes } =
            changes;

        if let Some(change) = client_change {
            self.client_config = change;
        }

        if let Some(change) = xdg_config_change {
            let node = self
                .tree
                .get_mut(self.xdg_config_node_id)
                .expect("client_config node should exist");
            node.get_mut().input = change;
            self.invalidate_subtree(self.xdg_config_node_id);
        }

        for change in ra_toml_changes {
            // turn and face the strain
            match change.change_kind {
                vfs::ChangeKind::Create => {
                    let input = parse_toml(change.file_id, vfs, &mut scratch_errors, errors)
                        .unwrap_or_default();
                    let _new_node = self.insert_toml(change.file_id, input);
                }
                vfs::ChangeKind::Modify => {
                    let input = parse_toml(change.file_id, vfs, &mut scratch_errors, errors)
                        .unwrap_or_default();
                    if let Err(e) = self.update_toml(change.file_id, input) {
                        errors.push(e);
                    }
                }
                vfs::ChangeKind::Delete => {
                    self.remove_toml(change.file_id);
                }
            }
        }
    }
}
