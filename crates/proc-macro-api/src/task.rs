//! task.rs is used to abstract ProcMacroServerProcess::send_task in process.rs
//! The ultimate goal is to encapsulate legacy_protocol and don't directly use legacy_protocol in process.rs
//!

use crate::ServerError;

pub trait TaskClient {
    type Task;
    type TaskResult;

    fn send_task(&mut self, task: Self::Task) -> Result<Self::TaskResult, ServerError>;
}
