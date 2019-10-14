use ra_db::{CrateId, Edition};

use crate::{db::DefDatabase, ids::ModuleId, name::AsName, Name};

#[derive(Debug)]
pub struct CrateDependency {
    pub krate: CrateId,
    pub name: Name,
}

pub(crate) fn crate_dependencies(db: &impl DefDatabase, crate_id: CrateId) -> Vec<CrateDependency> {
    db.crate_graph()
        .dependencies(crate_id)
        .map(|dep| {
            let krate = dep.crate_id();
            let name = dep.as_name();
            CrateDependency { krate, name }
        })
        .collect()
}

pub(crate) fn crate_root_module(db: &impl DefDatabase, crate_id: CrateId) -> Option<ModuleId> {
    let module_id = db.crate_def_map(crate_id).root();
    let module = ModuleId { krate: crate_id, module_id };
    Some(module)
}

pub(crate) fn crate_edition(db: &impl DefDatabase, crate_id: CrateId) -> Edition {
    db.crate_graph().edition(crate_id)
}
