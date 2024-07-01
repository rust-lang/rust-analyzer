//! A `cargo-metadata`-equivalent for non-Cargo build systems.
use std::{io, process::Command};

use crossbeam_channel::Sender;
use paths::{AbsPathBuf, Utf8Path};
use project_model::ProjectJsonData;
use serde::{Deserialize, Serialize};

use crate::command::{CommandHandle, ParseFromLine};

/// A command wrapper for getting a `rust-project.json`.
///
/// This is analogous to `cargo-metadata`, but for non-Cargo build systems.
pub struct JsonWorkspace {
    command: Vec<String>,
    sender: Sender<DiscoverProjectMessage>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonArguments {
    Path(
        #[serde(serialize_with = "serialize_abs_pathbuf")]
        #[serde(deserialize_with = "deserialize_abs_pathbuf")]
        AbsPathBuf,
    ),
    Label(String),
}

impl JsonWorkspace {
    /// Create a new [JsonWorkspace].
    pub fn new(sender: Sender<DiscoverProjectMessage>, command: Vec<String>) -> Self {
        Self { sender, command }
    }

    /// Spawn the command inside [JsonWorkspace] and report progress, if any.
    pub fn spawn(&self, arg: JsonArguments) -> io::Result<JsonWorkspaceHandle> {
        let command = &self.command[0];
        let args = &self.command[1..];

        let mut cmd = Command::new(command);
        cmd.args(args);

        let arg = serde_json::to_string(&arg)?;
        cmd.arg(arg);

        Ok(JsonWorkspaceHandle { _handle: CommandHandle::spawn(cmd, self.sender.clone())? })
    }
}

/// A handle to a spawned [JsonWorkspace].
#[derive(Debug)]
pub struct JsonWorkspaceHandle {
    _handle: CommandHandle<DiscoverProjectMessage>,
}

/// An enum containing either progress messages or the materialized rust-project.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DiscoverProjectMessage {
    Error { message: String, context: Option<String> },
    Progress { message: String },
    Finished(FinishedOutput),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct FinishedOutput {
    pub project: ProjectJsonData,
    #[serde(serialize_with = "serialize_abs_pathbuf")]
    #[serde(deserialize_with = "deserialize_abs_pathbuf")]
    pub buildfile: AbsPathBuf,
}

fn deserialize_abs_pathbuf<'de, D>(de: D) -> std::result::Result<AbsPathBuf, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let path = String::deserialize(de)?;

    AbsPathBuf::try_from(path.as_ref())
        .map_err(|err| serde::de::Error::custom(format!("invalid path name: {err:?}")))
}

fn serialize_abs_pathbuf<S>(path: &AbsPathBuf, se: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let path: &Utf8Path = path.as_ref();
    se.serialize_str(path.as_str())
}

impl ParseFromLine for DiscoverProjectMessage {
    fn from_line(line: &str, _error: &mut String) -> Option<Self> {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            return Some(DiscoverProjectMessage::Error { message: line.to_owned(), context: None });
        };

        if let Ok(project) = serde_json::from_value::<FinishedOutput>(value.clone()) {
            return Some(DiscoverProjectMessage::Finished(project));
        }

        if let Some(message) = value.pointer("/fields/message") {
            return Some(DiscoverProjectMessage::Progress {
                message: message.as_str().unwrap().to_owned(),
            });
        }

        if let Some(error) = value.pointer("/fields/error") {
            let context =
                value.pointer("/fields/source").map(|source| source.as_str().unwrap().to_owned());

            return Some(DiscoverProjectMessage::Error {
                message: error.as_str().unwrap().to_owned(),
                context,
            });
        }

        None
    }

    fn from_eof() -> Option<Self> {
        None
    }
}
