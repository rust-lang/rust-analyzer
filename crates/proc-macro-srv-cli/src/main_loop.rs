//! The main loop of the proc-macro server.
use std::io;

use crate::task_executor::{
    TaskExecutor, json_task_executor::JsonTaskExecutor,
    postcard_task_executor::PostcardTaskExecutor,
};

pub(crate) fn run() -> io::Result<()> {
    // Check environment variable or command line args to determine protocol
    let args: Vec<String> = std::env::args().collect();
    let mut format = None;

    // Parse --format argument
    for i in 1..args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            format = Some(args[i + 1].as_str());
            break;
        }
    }

    // Default to JSON for backward compatibility
    let protocol = format.unwrap_or("json");

    match protocol {
        "json" => {
            let executor = JsonTaskExecutor;
            executor.run()
        }
        "postcard" => {
            let executor = PostcardTaskExecutor;
            executor.run()
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported protocol format: {}. Supported formats: json, postcard", protocol),
        )),
    }
}
