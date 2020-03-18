//! Client-side Proc-Macro crate
//!
//! We separate proc-macro expanding logic to an extern program to allow
//! different implementations (e.g. wasm or dylib loading). And this crate
//! is used for provide basic infra-structure for commnicate between two
//! process: Client (RA itself), Server (the external program)

#![allow(unused_variables)]

mod rpc;

use ra_mbe::ProcMacroError;
use ra_tt::Subtree;
pub use rpc::{ExpansionResult, ExpansionTask};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcMacroProcessExpander {
    process_path: PathBuf,
}

impl ProcMacroProcessExpander {
    pub fn custom_derive(
        &self,
        dylib_path: &Path,
        subtree: &Subtree,
        derive_name: &str,
    ) -> Result<Subtree, ProcMacroError> {
        if self.process_path == PathBuf::default() {
            return Err(ProcMacroError::Dummy);
        }
        let process = match Command::new(self.process_path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(process) => process,
            Err(err) => Err(ProcMacroError::IOError(err))?,
        };

        let task = ExpansionTask {
            macro_body: subtree.clone(),
            macro_name: derive_name.to_string(),
            attributes: None,
            lib: dylib_path.to_path_buf(),
        };
        let data = serde_json::to_string(&task)
            .map_err(|err| ProcMacroError::JsonError(err.to_string()))?;
        if let Err(err) = process.stdin.unwrap().write_all(data.as_bytes()) {
            return Err(ProcMacroError::IOError(err));
        };

        let mut s = String::new();
        match process.stdout.unwrap().read_to_string(&mut s) {
            Err(err) => return Err(ProcMacroError::IOError(err)),
            Ok(_) => (),
        }
        let result: ExpansionResult =
            serde_json::from_str(&s).map_err(|err| ProcMacroError::JsonError(err.to_string()))?;
        match result {
            ExpansionResult::Success { expansion } => Ok(expansion),
            ExpansionResult::Error { reason } => Err(ProcMacroError::ExpansionError(reason)),
        }
    }
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
    ) -> Result<Subtree, ProcMacroError> {
        self.expander.custom_derive(&self.dylib_path, subtree, derive_name)
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
        let expander = ProcMacroProcessExpander { process_path: PathBuf::default() };
        ProcMacroClient { expander: Arc::new(expander) }
    }

    pub fn by_dylib_path(&self, dylib_path: &Path) -> ProcMacro {
        ProcMacro { expander: self.expander.clone(), dylib_path: dylib_path.into() }
    }
}
