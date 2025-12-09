use ide_db::{
    FileId, FxIndexSet, RootDatabase,
    base_db::{CrateOrigin, RootQueryDb},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub root_file_id: FileId,
}

// Feature: Show Dependency Tree
//
// Shows a view tree with all the dependencies of this project
//
// | Editor  | Panel Name |
// |---------|------------|
// | VS Code | **Rust Dependencies** |
//
// ![Show Dependency Tree](https://user-images.githubusercontent.com/5748995/229394139-2625beab-f4c9-484b-84ed-ad5dee0b1e1a.png)
pub(crate) fn fetch_crates(db: &RootDatabase) -> FxIndexSet<CrateInfo> {
    db.all_crates()
        .iter()
        .copied()
        .map(|crate_id| (crate_id, crate_id.data(db), crate_id.extra_data(db)))
        .filter(|(_, data, _)| !matches!(data.origin, CrateOrigin::Local { .. }))
        .map(|(crate_id, _, extra_data)| crate_info(db, crate_id, extra_data))
        .collect()
}

fn crate_info(
    db: &RootDatabase,
    crate_id: ide_db::base_db::Crate,
    extra_data: &ide_db::base_db::ExtraCrateData,
) -> CrateInfo {
    let crate_name = crate_name(extra_data);
    let version = extra_data.version.clone();
    CrateInfo { name: crate_name, version, root_file_id: crate_id.root_file(db) }
}

fn crate_name(data: &ide_db::base_db::ExtraCrateData) -> Option<String> {
    data.display_name.as_ref().map(|it| it.canonical_name().as_str().to_owned())
}
