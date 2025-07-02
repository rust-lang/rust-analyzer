//! TaskExecutor trait and implementations for server-side protocol handling
//! This mirrors the client-side TaskClient pattern

use std::io;

pub(crate) mod json_task_executor;
#[allow(dead_code)]
pub(crate) mod postcard_task_executor;

/// Generic trait for executing task processing loops on the server side
pub(crate) trait TaskExecutor {
    /// Run the main server loop for processing client requests
    fn run(&self) -> io::Result<()>;
}
