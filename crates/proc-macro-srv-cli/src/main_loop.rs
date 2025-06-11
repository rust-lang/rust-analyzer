//! The main loop of the proc-macro server.
use std::io;

use crate::task_executor::{TaskExecutor, json_task_executor::JsonTaskExecutor};

pub(crate) fn run() -> io::Result<()> {
    // Use JsonTaskExecutor for legacy protocol support
    let executor = JsonTaskExecutor;
    executor.run()
}
