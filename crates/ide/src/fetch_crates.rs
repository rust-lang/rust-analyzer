use ide_db::{
    base_db::{CrateOrigin, FileId, SourceDatabase},
    FxIndexSet, RootDatabase,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub root_file_id: FileId,
    pub manifest_path_id: Option<FileId>,
}

// Feature: Show Dependency Tree
//
// Shows a view tree with all the dependencies of this project
//
// |===
// | Editor  | Panel Name
//
// | VS Code | **Rust Dependencies**
// |===
//
// image::https://user-images.githubusercontent.com/5748995/229394139-2625beab-f4c9-484b-84ed-ad5dee0b1e1a.png[]
pub(crate) fn fetch_crates(db: &RootDatabase) -> FxIndexSet<CrateInfo> {
    let crate_graph = db.crate_graph();
    crate_graph
        .iter()
        .map(|crate_id| &crate_graph[crate_id])
        .filter(|&data| !matches!(data.origin, CrateOrigin::Local { .. }))
        .map(|data| crate_info(data))
        .collect()
}

fn crate_info(data: &ide_db::base_db::CrateData) -> CrateInfo {
    let name = data.display_name.as_ref().map(|it| it.canonical_name().to_owned());

    CrateInfo {
        name,
        version: data.version.clone(),
        root_file_id: data.root_file_id,
        manifest_path_id: data.manifest_path_id,
    }
}
