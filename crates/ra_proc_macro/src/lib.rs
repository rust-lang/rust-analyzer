//! Client-side Proc-Macro crate
//!
//! We separate proc-macro expanding logic to an extern program to allow
//! different implementations (e.g. wasm or dylib loading). And this crate
//! is used for provide basic infra-structure for commnicate between two
//! process: Client (RA itself), Server (the external program)

#![allow(unused_variables)]

use ra_mbe::ExpandError;
use ra_tt::Subtree;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcMacroProcessExpander {
    process_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcMacro {
    expander: Arc<ProcMacroProcessExpander>,
    dylib_path: PathBuf,
}

impl ProcMacro {
    pub fn custom_derive(
        &self,
        subtree: &Subtree,
        derive_name: &str,
    ) -> Result<Subtree, ExpandError> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcMacroClient {
    expander: Arc<ProcMacroProcessExpander>,
}

impl ProcMacroClient {
    pub fn extern_process(process_path: &Path) -> ProcMacroClient {
        let expander = ProcMacroProcessExpander { process_path: process_path.into() };
        ProcMacroClient { expander: Arc::new(expander) }
    }

    pub fn dummy() -> ProcMacroClient {
        let expander = ProcMacroProcessExpander { process_path: "".into() };
        ProcMacroClient { expander: Arc::new(expander) }
    }

    pub fn by_dylib_path(&self, dylib_path: &Path) -> ProcMacro {
        ProcMacro { expander: self.expander.clone(), dylib_path: dylib_path.into() }
    }
}
