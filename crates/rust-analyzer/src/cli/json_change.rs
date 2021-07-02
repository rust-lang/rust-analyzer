//! Fully type-check project and print various stats, like the number of type
//! errors.

use ide::Change;
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace};
use std::path::Path;

use crate::cli::{load_cargo::LoadCargoConfig, Result};

use vfs::{AbsPath, AbsPathBuf};

use std::fs;

use crate::cli::load_cargo::load_change;

pub struct JsonChangeCmd {}

impl JsonChangeCmd {
    /// Execute with e.g.
    /// ```no_compile
    /// cargo run --bin rust-analyzer json-change ../ink/examples/flipper/Cargo.toml
    /// ```
    pub fn run(self, root: &Path) -> Result<()> {
        let change = get_change_data(root, &|_| {})?;
        let json = serde_json::to_string(&change).expect("serialization of change must work");
        fs::write("./change.json", json).expect("Unable to write file");
        Ok(())
    }
}

fn get_change_data(root: &Path, progress: &dyn Fn(String)) -> Result<Change> {
    let mut cargo_config = CargoConfig::default();
    cargo_config.no_sysroot = false;
    let root = AbsPathBuf::assert(std::env::current_dir()?.join(root));

    let root = AbsPath::assert(&root);
    let root = ProjectManifest::discover_single(root)?;
    let ws = ProjectWorkspace::load(root, &cargo_config, &|_| {})?;

    let config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        wrap_rustc: true,
        with_proc_macro: false,
        prefill_caches: false,
    };

    let (change, _, _) = load_change(ws, &config, progress)?;

    Ok(change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_serialize_deserialize_change() -> Result<()> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let change = get_change_data(path, &|_| {})?;
        let json = serde_json::to_string(&change)?;
        let deserialized_change: Change = serde_json::from_str(&json)?;
        assert_eq!(change, deserialized_change);
        Ok(())
    }
}
