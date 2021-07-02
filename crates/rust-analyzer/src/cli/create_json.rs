//! Fully type-check project and print various stats, like the number of type
//! errors.

use crossbeam_channel::unbounded;
use ide::Change;
use project_model::{
    BuildDataCollector, CargoConfig, ProcMacroClient, ProjectManifest, ProjectWorkspace,
};
use std::path::Path;

use crate::cli::{load_cargo::LoadCargoConfig, Result};

use crate::reload::ProjectFolders;

use vfs::{loader::Handle, AbsPath, AbsPathBuf};

use std::fs;

use crate::cli::load_cargo::load_crate_graph;

pub struct CreateJsonCmd {}

impl CreateJsonCmd {
    /// Execute with e.g.
    /// ```no_compile
    /// cargo run --bin rust-analyzer create-json ../ink/examples/flipper/Cargo.toml
    /// ```
    pub fn run(self, root: &Path) -> Result<()> {
        let change = get_crate_data(root, &|_| {})?;

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

fn get_crate_data(root: &Path, progress: &dyn Fn(String)) -> Result<Change> {
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
    let (sender, receiver) = unbounded();
    let mut vfs = vfs::Vfs::default();
    let mut loader = {
        let loader =
            vfs_notify::NotifyHandle::spawn(Box::new(move |msg| sender.send(msg).unwrap()));
        Box::new(loader)
    };

    let proc_macro_client = if config.with_proc_macro {
        let path = std::env::current_exe()?;
        Some(ProcMacroClient::extern_process(path, &["proc-macro"]).unwrap())
    } else {
        None
    };

    let build_data = if config.load_out_dirs_from_check {
        let mut collector = BuildDataCollector::new(config.wrap_rustc);
        ws.collect_build_data_configs(&mut collector);
        Some(collector.collect(progress)?)
    } else {
        None
    };

    let crate_graph = ws.to_crate_graph(
        build_data.as_ref(),
        proc_macro_client.as_ref(),
        &mut |path: &AbsPath| {
            let contents = loader.load_sync(path);
            let path = vfs::VfsPath::from(path.to_path_buf());
            vfs.set_file_contents(path.clone(), contents);
            vfs.file_id(&path)
        },
    );

    let project_folders = ProjectFolders::new(&[ws], &[], build_data.as_ref());
    loader.set_config(vfs::loader::Config {
        load: project_folders.load,
        watch: vec![],
        version: 0,
    });

    let change = load_crate_graph(
        crate_graph.clone(),
        project_folders.source_root_config,
        &mut vfs,
        &receiver,
    );

    Ok(change)
}
