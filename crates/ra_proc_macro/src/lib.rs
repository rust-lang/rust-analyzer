//! Client-side Proc-Macro crate
//!
//! We separate proc-macro expanding logic to an extern program to allow
//! different implementations (e.g. wasm or dylib loading). And this crate
//! is used for provide basic infra-structure for commnicate between two
//! process: Client (RA itself), Server (the external program)
mod rpc;
mod process;
pub mod msg;

use process::ProcMacroProcessExpander;
use ra_mbe::ProcMacroError;
use ra_tt::Subtree;
pub use rpc::{ExpansionResult, ExpansionTask};

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct ProcMacro {
    expander: Arc<ProcMacroProcessExpander>,
    dylib_path: PathBuf,
}

impl Eq for ProcMacro {}
impl PartialEq for ProcMacro {
    fn eq(&self, other: &ProcMacro) -> bool {
        Arc::ptr_eq(&self.expander, &other.expander) && self.dylib_path == other.dylib_path
    }
}

impl ProcMacro {
    pub fn custom_derive(
        &self,
        subtree: &Subtree,
        derive_name: &str,
    ) -> Result<Subtree, ProcMacroError> {
        self.expander.custom_derive(&self.dylib_path, subtree, derive_name)
    }
}

#[derive(Debug)]
pub struct ProcMacroClient {
    expander: Arc<ProcMacroProcessExpander>,
}

impl ProcMacroClient {
    pub fn extern_process(process_path: &Path) -> Result<ProcMacroClient, std::io::Error> {
        let expander = ProcMacroProcessExpander::run(process_path)?;
        Ok(ProcMacroClient { expander: Arc::new(expander) })
    }

    pub fn dummy() -> ProcMacroClient {
        let expander = ProcMacroProcessExpander::default();
        ProcMacroClient { expander: Arc::new(expander) }
    }

    pub fn by_dylib_path(&self, dylib_path: &Path) -> ProcMacro {
        ProcMacro { expander: self.expander.clone(), dylib_path: dylib_path.into() }
    }
}
