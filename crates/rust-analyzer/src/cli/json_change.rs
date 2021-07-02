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

        // let (_, change2) = get_crate_data(root, &|_| {})?;

        // let _json =
        //    serde_json::to_string(&crate_graph).expect("serialization of crate_graph must work");

        let json = serde_json::to_string(&change).expect("serialization of change must work");
        /*
        _let deserialized_change: Change = serde_json::from_str(&json).expect("`Change` deserialization must work");
        // let json = str::replace(&json,  "'","@@@");
        let file_id = FileId(182);
        let mut host = AnalysisHost::new(None);
        host.apply_change(deserialized_change);
        let analysis = host.analysis();
        println!("getting status");
        let status = analysis.status(Some(file_id)).unwrap();
        println!("{}", status);
        let _config = DiagnosticsConfig::default();
        let _highlights: Vec<_> = analysis
            .highlight(file_id)
            .unwrap()
            .into_iter()
            .collect();
        // let _highlights = analysis.highlight(file_id);
        */

        fs::write("./change.json", json).expect("Unable to write file");

        // println!("{}", json);

        /*  let mut host = AnalysisHost::new(None);
        host.apply_change(change);
        let analysis = host.analysis();
        let file_id = FileId(0);
        */
        // let _highlights = analysis.highlight(file_id);
        // println!("{}", json);

        // let deserialized_change: Change = serde_json::from_str(&json).expect("`Change` deserialization must work");

        // println!("change_json:\n{}", change_json);

        // deserialize from json string
        /*
        let deserialized_crate_graph: CrateGraph =
            serde_json::from_str(&json).expect("deserialization must work");
        assert_eq!(
            crate_graph, deserialized_crate_graph,
            "Deserialized `CrateGraph` is not equal!"
        );
        */

        // Missing: Create a new `Change` object.
        //
        // `serde::Serialize` and `serde::Deserialize` are already supported by `Change`.
        // So this should work out of the box after the object has been created:
        //
        // ```
        // let json = serde_json::to_string(&change).expect("`Change` serialization must work");
        // println!("change json:\n{}", json);
        // let deserialized_change: Change = serde_json::from_str(&json).expect("`Change` deserialization must work");
        // assert_eq!(change.roots, deserialized_change.roots, "Deserialized `Change.roots` is not equal!");
        // assert_eq!(change.files_changed, deserialized_change.files_changed, "Deserialized `Change.roots` is not equal!");
        // ```

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
