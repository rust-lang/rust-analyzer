//! Flycheck provides the functionality needed to run `cargo check` to provide
//! LSP diagnostics based on the output of the command.

use std::{fmt, io, time::Duration};

use cargo_metadata::PackageId;
use command::{check_command, FlycheckConfig, Target};
use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use ide_db::FxHashSet;
use paths::AbsPathBuf;
use serde::Deserialize as _;
use serde_derive::Deserialize;

pub(crate) use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
};
use triomphe::Arc;

use crate::command::{CommandHandle, ParseFromLine};

pub(crate) mod command;

/// Flycheck wraps the shared state and communication machinery used for
/// running `cargo check` (or other compatible command) and providing
/// diagnostics based on the output.
/// The spawned thread is shut down when this struct is dropped.
#[derive(Debug)]
pub(crate) struct FlycheckHandle {
    // XXX: drop order is significant
    sender: Sender<StateChange>,
    _thread: stdx::thread::JoinHandle,
    id: usize,
}

impl FlycheckHandle {
    pub(crate) fn spawn(
        id: usize,
        sender: Sender<FlycheckMessage>,
        sysroot_root: Option<AbsPathBuf>,
        workspace_root: AbsPathBuf,
        manifest_path: Option<AbsPathBuf>,
    ) -> FlycheckHandle {
        let actor = FlycheckActor::new(id, sender, sysroot_root, workspace_root, manifest_path);
        let (sender, receiver) = unbounded::<StateChange>();
        let thread = stdx::thread::Builder::new(stdx::thread::ThreadIntent::Worker)
            .name("Flycheck".to_owned())
            .spawn(move || actor.run(receiver))
            .expect("failed to spawn thread");
        FlycheckHandle { id, sender, _thread: thread }
    }

    /// Schedule a re-start of the cargo check worker to do a workspace wide check.
    pub(crate) fn restart_workspace(&self, saved_file: Option<AbsPathBuf>, config: FlycheckConfig) {
        self.sender
            .send(StateChange::Restart { package: None, saved_file, target: None, config })
            .unwrap();
    }

    /// Schedule a re-start of the cargo check worker to do a package wide check.
    pub(crate) fn restart_for_package(
        &self,
        package: String,
        target: Option<Target>,
        config: FlycheckConfig,
    ) {
        self.sender
            .send(StateChange::Restart { package: Some(package), saved_file: None, target, config })
            .unwrap();
    }

    /// Stop this cargo check worker.
    pub(crate) fn cancel(&self) {
        self.sender.send(StateChange::Cancel).unwrap();
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }
}

pub(crate) enum FlycheckMessage {
    /// Request adding a diagnostic with fixes included to a file
    AddDiagnostic {
        id: usize,
        workspace_root: Arc<AbsPathBuf>,
        diagnostic: Diagnostic,
        package_id: Option<Arc<PackageId>>,
    },

    /// Request clearing all outdated diagnostics.
    ClearDiagnostics {
        id: usize,
        /// The package whose diagnostics to clear, or if unspecified, all diagnostics.
        package_id: Option<Arc<PackageId>>,
    },

    /// Request check progress notification to client
    Progress {
        /// Flycheck instance ID
        id: usize,
        progress: Progress,
    },
}

impl fmt::Debug for FlycheckMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlycheckMessage::AddDiagnostic { id, workspace_root, diagnostic, package_id } => f
                .debug_struct("AddDiagnostic")
                .field("id", id)
                .field("workspace_root", workspace_root)
                .field("package_id", package_id)
                .field("diagnostic_code", &diagnostic.code.as_ref().map(|it| &it.code))
                .finish(),
            FlycheckMessage::ClearDiagnostics { id, package_id } => f
                .debug_struct("ClearDiagnostics")
                .field("id", id)
                .field("package_id", package_id)
                .finish(),
            FlycheckMessage::Progress { id, progress } => {
                f.debug_struct("Progress").field("id", id).field("progress", progress).finish()
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum Progress {
    DidStart,
    DidCheckCrate(String),
    DidFinish(io::Result<()>),
    DidCancel,
    DidFailToRestart(String),
}

enum StateChange {
    Restart {
        package: Option<String>,
        saved_file: Option<AbsPathBuf>,
        target: Option<Target>,
        config: FlycheckConfig,
    },
    Cancel,
}

/// A [`FlycheckActor`] is a single check instance of a workspace.
struct FlycheckActor {
    /// The workspace id of this flycheck instance.
    id: usize,

    sender: Sender<FlycheckMessage>,
    manifest_path: Option<AbsPathBuf>,
    /// Either the workspace root of the workspace we are flychecking,
    /// or the project root of the project.
    root: Arc<AbsPathBuf>,
    sysroot_root: Option<AbsPathBuf>,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo check` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    command_handle: Option<CommandHandle<CargoCheckMessage>>,
    /// The receiver side of the channel mentioned above.
    command_receiver: Option<Receiver<CargoCheckMessage>>,
    diagnostics_cleared_for: FxHashSet<Arc<PackageId>>,
    diagnostics_cleared_for_all: bool,
    diagnostics_received: bool,
}

#[allow(clippy::large_enum_variant)]
enum Event {
    RequestStateChange(StateChange),
    CheckEvent(Option<CargoCheckMessage>),
}

impl FlycheckActor {
    fn new(
        id: usize,
        sender: Sender<FlycheckMessage>,
        sysroot_root: Option<AbsPathBuf>,
        workspace_root: AbsPathBuf,
        manifest_path: Option<AbsPathBuf>,
    ) -> FlycheckActor {
        tracing::info!(%id, ?workspace_root, "Spawning flycheck");
        FlycheckActor {
            id,
            sender,
            sysroot_root,
            root: Arc::new(workspace_root),
            manifest_path,
            command_handle: None,
            command_receiver: None,
            diagnostics_cleared_for: Default::default(),
            diagnostics_cleared_for_all: false,
            diagnostics_received: false,
        }
    }

    fn report_progress(&self, progress: Progress) {
        self.send(FlycheckMessage::Progress { id: self.id, progress });
    }

    fn next_event(&self, inbox: &Receiver<StateChange>) -> Option<Event> {
        let Some(command_receiver) = &self.command_receiver else {
            return inbox.recv().ok().map(Event::RequestStateChange);
        };

        // Biased to give restarts a preference so check outputs don't block a restart or stop
        select_biased! {
            recv(inbox) -> msg => msg.ok().map(Event::RequestStateChange),
            recv(command_receiver) -> msg => Some(Event::CheckEvent(msg.ok())),
        }
    }

    fn run(mut self, inbox: Receiver<StateChange>) {
        'event: while let Some(event) = self.next_event(&inbox) {
            match event {
                Event::RequestStateChange(StateChange::Cancel) => {
                    tracing::debug!(flycheck_id = self.id, "flycheck cancelled");
                    self.cancel_check_process();
                }
                Event::RequestStateChange(StateChange::Restart {
                    package,
                    saved_file,
                    target,
                    config,
                }) => {
                    // Cancel the previously spawned process
                    self.cancel_check_process();
                    while let Ok(restart) = inbox.recv_timeout(Duration::from_millis(50)) {
                        // restart chained with a stop, so just cancel
                        if let StateChange::Cancel = restart {
                            continue 'event;
                        }
                    }

                    let Some(command) = check_command(
                        &self.root,
                        &self.sysroot_root,
                        &self.manifest_path,
                        config,
                        package.as_deref(),
                        saved_file.as_deref(),
                        target,
                    ) else {
                        continue;
                    };

                    let formatted_command = format!("{command:?}");

                    tracing::debug!(?command, "will restart flycheck");
                    let (sender, receiver) = unbounded();
                    match CommandHandle::spawn(command, sender) {
                        Ok(command_handle) => {
                            tracing::debug!(command = formatted_command, "did restart flycheck");
                            self.command_handle = Some(command_handle);
                            self.command_receiver = Some(receiver);
                            self.report_progress(Progress::DidStart);
                        }
                        Err(error) => {
                            self.report_progress(Progress::DidFailToRestart(format!(
                                "Failed to run the following command: {formatted_command} error={error}"
                            )));
                        }
                    }
                }
                Event::CheckEvent(None) => {
                    tracing::debug!(flycheck_id = self.id, "flycheck finished");

                    // Watcher finished
                    let command_handle = self.command_handle.take().unwrap();
                    self.command_receiver.take();
                    let formatted_handle = format!("{command_handle:?}");

                    let res = command_handle.join();
                    if let Err(error) = &res {
                        tracing::error!(
                            "Flycheck failed to run the following command: {}, error={}",
                            formatted_handle,
                            error
                        );
                    }
                    if !self.diagnostics_received {
                        tracing::trace!(flycheck_id = self.id, "clearing diagnostics");
                        // We finished without receiving any diagnostics.
                        // Clear everything for good measure
                        self.send(FlycheckMessage::ClearDiagnostics {
                            id: self.id,
                            package_id: None,
                        });
                    }
                    self.clear_diagnostics_state();

                    self.report_progress(Progress::DidFinish(res));
                }
                Event::CheckEvent(Some(message)) => match message {
                    CargoCheckMessage::CompilerArtifact(msg) => {
                        tracing::trace!(
                            flycheck_id = self.id,
                            artifact = msg.target.name,
                            package_id = msg.package_id.repr,
                            "artifact received"
                        );
                        self.report_progress(Progress::DidCheckCrate(msg.target.name));
                        let package_id = Arc::new(msg.package_id);
                        if self.diagnostics_cleared_for.insert(package_id.clone()) {
                            tracing::trace!(
                                flycheck_id = self.id,
                                package_id = package_id.repr,
                                "clearing diagnostics"
                            );
                            self.send(FlycheckMessage::ClearDiagnostics {
                                id: self.id,
                                package_id: Some(package_id),
                            });
                        }
                    }
                    CargoCheckMessage::Diagnostic { diagnostic, package_id } => {
                        tracing::trace!(
                            flycheck_id = self.id,
                            message = diagnostic.message,
                            package_id = package_id.as_ref().map(|it| &it.repr),
                            "diagnostic received"
                        );
                        self.diagnostics_received = true;
                        if let Some(package_id) = &package_id {
                            if self.diagnostics_cleared_for.insert(package_id.clone()) {
                                tracing::trace!(
                                    flycheck_id = self.id,
                                    package_id = package_id.repr,
                                    "clearing diagnostics"
                                );
                                self.send(FlycheckMessage::ClearDiagnostics {
                                    id: self.id,
                                    package_id: Some(package_id.clone()),
                                });
                            }
                        } else if !self.diagnostics_cleared_for_all {
                            self.diagnostics_cleared_for_all = true;
                            self.send(FlycheckMessage::ClearDiagnostics {
                                id: self.id,
                                package_id: None,
                            });
                        }
                        self.send(FlycheckMessage::AddDiagnostic {
                            id: self.id,
                            package_id,
                            workspace_root: self.root.clone(),
                            diagnostic,
                        });
                    }
                },
            }
        }
        // If we rerun the thread, we need to discard the previous check results first
        self.cancel_check_process();
    }

    fn cancel_check_process(&mut self) {
        if let Some(command_handle) = self.command_handle.take() {
            tracing::debug!(
                command = ?command_handle,
                "did  cancel flycheck"
            );
            command_handle.cancel();
            self.command_receiver.take();
            self.report_progress(Progress::DidCancel);
        }
        self.clear_diagnostics_state();
    }

    fn clear_diagnostics_state(&mut self) {
        self.diagnostics_cleared_for.clear();
        self.diagnostics_cleared_for_all = false;
        self.diagnostics_received = false;
    }

    #[track_caller]
    fn send(&self, check_task: FlycheckMessage) {
        self.sender.send(check_task).unwrap();
    }
}

#[allow(clippy::large_enum_variant)]
enum CargoCheckMessage {
    CompilerArtifact(cargo_metadata::Artifact),
    Diagnostic { diagnostic: Diagnostic, package_id: Option<Arc<PackageId>> },
}

impl ParseFromLine for CargoCheckMessage {
    fn from_line(line: &str, error: &mut String) -> Option<Self> {
        let mut deserializer = serde_json::Deserializer::from_str(line);
        deserializer.disable_recursion_limit();
        if let Ok(message) = JsonMessage::deserialize(&mut deserializer) {
            return match message {
                // Skip certain kinds of messages to only spend time on what's useful
                JsonMessage::Cargo(message) => match message {
                    cargo_metadata::Message::CompilerArtifact(artifact) if !artifact.fresh => {
                        Some(CargoCheckMessage::CompilerArtifact(artifact))
                    }
                    cargo_metadata::Message::CompilerMessage(msg) => {
                        Some(CargoCheckMessage::Diagnostic {
                            diagnostic: msg.message,
                            package_id: Some(Arc::new(msg.package_id)),
                        })
                    }
                    _ => None,
                },
                JsonMessage::Rustc(message) => {
                    Some(CargoCheckMessage::Diagnostic { diagnostic: message, package_id: None })
                }
            };
        }

        error.push_str(line);
        error.push('\n');
        None
    }

    fn from_eof() -> Option<Self> {
        None
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum JsonMessage {
    Cargo(cargo_metadata::Message),
    Rustc(Diagnostic),
}
