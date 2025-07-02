//! Handle process life-time and message passing for proc-macro client

use std::{
    io::{self, BufReader, Read},
    panic::AssertUnwindSafe,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Mutex, OnceLock},
};

use paths::AbsPath;
use stdx::JodChild;

use crate::{
    ProcMacroKind, ServerError,
    legacy_protocol::{
        msg::{
            CURRENT_API_VERSION, RUST_ANALYZER_SPAN_SUPPORT, Request, Response, ServerConfig,
            SpanMode,
        },
        task_impl::JsonTaskClient,
    },
    task::TaskClient,
};

/// Represents a process handling proc-macro communication.
#[derive(Debug)]
pub(crate) struct ProcMacroServerProcess {
    /// The state of the proc-macro server process, the protocol is currently strictly sequential
    /// hence the lock on the state.
    state: Mutex<ProcessSrvState>,
    version: u32,
    mode: SpanMode,
    /// Populated when the server exits.
    exited: OnceLock<AssertUnwindSafe<ServerError>>,
}

/// Maintains the state of the proc-macro server process.
#[derive(Debug)]
struct ProcessSrvState {
    process: Process,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl ProcMacroServerProcess {
    /// Starts the proc-macro server and performs a version check
    pub(crate) fn run<'a>(
        process_path: &AbsPath,
        env: impl IntoIterator<
            Item = (impl AsRef<std::ffi::OsStr>, &'a Option<impl 'a + AsRef<std::ffi::OsStr>>),
        > + Clone,
    ) -> io::Result<ProcMacroServerProcess> {
        let create_srv = || {
            let mut process = Process::run(process_path, env.clone())?;
            let (stdin, stdout) = process.stdio().expect("couldn't access child stdio");

            io::Result::Ok(ProcMacroServerProcess {
                state: Mutex::new(ProcessSrvState { process, stdin, stdout }),
                version: 0,
                mode: SpanMode::Id,
                exited: OnceLock::new(),
            })
        };
        let mut srv = create_srv()?;
        tracing::info!("sending proc-macro server version check");
        match srv.version_check() {
            Ok(v) if v > CURRENT_API_VERSION => Err(io::Error::other(
                format!( "The version of the proc-macro server ({v}) in your Rust toolchain is newer than the version supported by your rust-analyzer ({CURRENT_API_VERSION}).
            This will prevent proc-macro expansion from working. Please consider updating your rust-analyzer to ensure compatibility with your current toolchain."
                ),
            )),
            Ok(v) => {
                tracing::info!("Proc-macro server version: {v}");
                srv.version = v;
                if srv.version >= RUST_ANALYZER_SPAN_SUPPORT {
                    if let Ok(mode) = srv.enable_rust_analyzer_spans() {
                        srv.mode = mode;
                    }
                }
                tracing::info!("Proc-macro server span mode: {:?}", srv.mode);
                Ok(srv)
            }
            Err(e) => {
                tracing::info!(%e, "proc-macro version check failed");
                Err(
                    io::Error::other(format!("proc-macro server version check failed: {e}")),
                )
            }
        }
    }

    /// Returns the server error if the process has exited.
    pub(crate) fn exited(&self) -> Option<&ServerError> {
        self.exited.get().map(|it| &it.0)
    }

    /// Retrieves the API version of the proc-macro server.
    pub(crate) fn version(&self) -> u32 {
        self.version
    }

    /// Checks the API version of the running proc-macro server.
    fn version_check(&self) -> Result<u32, ServerError> {
        let request = Request::ApiVersionCheck {};
        let response = self.send_task(request)?;

        match response {
            Response::ApiVersionCheck(version) => Ok(version),
            _ => Err(ServerError { message: "unexpected response".to_owned(), io: None }),
        }
    }

    /// Enable support for rust-analyzer span mode if the server supports it.
    fn enable_rust_analyzer_spans(&self) -> Result<SpanMode, ServerError> {
        let request = Request::SetConfig(ServerConfig { span_mode: SpanMode::RustAnalyzer });
        let response = self.send_task(request)?;

        match response {
            Response::SetConfig(ServerConfig { span_mode }) => Ok(span_mode),
            _ => Err(ServerError { message: "unexpected response".to_owned(), io: None }),
        }
    }

    /// Finds proc-macros in a given dynamic library.
    pub(crate) fn find_proc_macros(
        &self,
        dylib_path: &AbsPath,
    ) -> Result<Result<Vec<(String, ProcMacroKind)>, String>, ServerError> {
        let request = Request::ListMacros { dylib_path: dylib_path.to_path_buf().into() };

        let response = self.send_task(request)?;

        match response {
            Response::ListMacros(it) => Ok(it),
            _ => Err(ServerError { message: "unexpected response".to_owned(), io: None }),
        }
    }

    /// Sends a request to the proc-macro server and waits for a response.
    pub(crate) fn send_task(&self, req: Request) -> Result<Response, ServerError> {
        if let Some(server_error) = self.exited.get() {
            return Err(server_error.0.clone());
        }

        let state = &mut *self.state.lock().unwrap();
        // Check environment variable to determine which protocol to use
        let protocol = std::env::var("RUST_ANALYZER_PROC_MACRO_PROTOCOL")
            .unwrap_or_else(|_| "json".to_owned());

        let result = match protocol.as_str() {
            "postcard" => {
                tracing::warn!("Postcard protocol requested but not fully implemented, using JSON");

                let mut buf = String::new();
                let mut client = JsonTaskClient {
                    writer: &mut state.stdin,
                    reader: &mut state.stdout,
                    buf: &mut buf,
                };
                client.send_task(req)
            }
            _ => {
                // Default to JSON protocol
                let mut buf = String::new();
                let mut client = JsonTaskClient {
                    writer: &mut state.stdin,
                    reader: &mut state.stdout,
                    buf: &mut buf,
                };
                client.send_task(req)
            }
        };

        result.map_err(|e| {
            if e.io.as_ref().map(|it| it.kind()) == Some(io::ErrorKind::BrokenPipe) {
                match state.process.child.try_wait() {
                    Ok(None) | Err(_) => e,
                    Ok(Some(status)) => {
                        let mut msg = String::new();
                        if !status.success() {
                            if let Some(stderr) = state.process.child.stderr.as_mut() {
                                _ = stderr.read_to_string(&mut msg);
                            }
                        }
                        let server_error = ServerError {
                            message: format!(
                                "proc-macro server exited with {status}{}{msg}",
                                if msg.is_empty() { "" } else { ": " }
                            ),
                            io: None,
                        };
                        // `AssertUnwindSafe` is fine here, we already correct initialized
                        // server_error at this point.
                        self.exited.get_or_init(|| AssertUnwindSafe(server_error)).0.clone()
                    }
                }
            } else {
                e
            }
        })
    }
}

/// Manages the execution of the proc-macro server process.
#[derive(Debug)]
struct Process {
    child: JodChild,
}

impl Process {
    /// Runs a new proc-macro server process with the specified environment variables.
    fn run<'a>(
        path: &AbsPath,
        env: impl IntoIterator<
            Item = (impl AsRef<std::ffi::OsStr>, &'a Option<impl 'a + AsRef<std::ffi::OsStr>>),
        >,
    ) -> io::Result<Process> {
        let child = JodChild(mk_child(path, env)?);
        Ok(Process { child })
    }

    /// Retrieves stdin and stdout handles for the process.
    fn stdio(&mut self) -> Option<(ChildStdin, BufReader<ChildStdout>)> {
        let stdin = self.child.stdin.take()?;
        let stdout = self.child.stdout.take()?;
        let read = BufReader::new(stdout);

        Some((stdin, read))
    }
}

/// Creates and configures a new child process for the proc-macro server.
fn mk_child<'a>(
    path: &AbsPath,
    extra_env: impl IntoIterator<
        Item = (impl AsRef<std::ffi::OsStr>, &'a Option<impl 'a + AsRef<std::ffi::OsStr>>),
    >,
) -> io::Result<Child> {
    #[allow(clippy::disallowed_methods)]
    let mut cmd = Command::new(path);

    // Check for protocol selection environment variable
    if let Ok(protocol) = std::env::var("RUST_ANALYZER_PROC_MACRO_PROTOCOL") {
        match protocol.as_str() {
            "postcard" => {
                cmd.args(["--format", "postcard"]);
            }
            "json" => {
                cmd.args(["--format", "json"]);
            }
            _ => {
                tracing::warn!("Unknown protocol '{}', defaulting to json", protocol);
                cmd.args(["--format", "json"]);
            }
        }
    } else {
        // Default to JSON protocol for backward compatibility
        cmd.args(["--format", "json"]);
    }

    for env in extra_env {
        match env {
            (key, Some(val)) => cmd.env(key, val),
            (key, None) => cmd.env_remove(key),
        };
    }
    cmd.env("RUST_ANALYZER_INTERNALS_DO_NOT_USE", "this is unstable")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if cfg!(windows) {
        let mut path_var = std::ffi::OsString::new();
        path_var.push(path.parent().unwrap().parent().unwrap());
        path_var.push("\\bin;");
        path_var.push(std::env::var_os("PATH").unwrap_or_default());
        cmd.env("PATH", path_var);
    }
    cmd.spawn()
}
