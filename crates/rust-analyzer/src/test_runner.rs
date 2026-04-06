//! This module provides the functionality needed to run `cargo test` in a background
//! thread and report the result of each test in a channel.

use std::process::Command;

use crossbeam_channel::Sender;
use paths::{AbsPath, Utf8Path};
use project_model::TargetKind;
use serde::Deserialize as _;
use serde_derive::Deserialize;
use toolchain::Tool;

use crate::{
    command::{CommandHandle, JsonLinesParser},
    config::TestRunnerKind,
    flycheck::CargoOptions,
};

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub(crate) enum TestState {
    Started,
    Ok,
    Ignored,
    Failed {
        // the stdout field is not always present depending on cargo test flags
        #[serde(skip_serializing_if = "String::is_empty", default)]
        stdout: String,
    },
}

#[derive(Debug)]
pub(crate) struct CargoTestMessage {
    pub target: TestTarget,
    pub output: CargoTestOutput,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum CargoTestOutput {
    Test {
        name: String,
        #[serde(flatten)]
        state: TestState,
    },
    Suite,
    Finished,
    Custom {
        text: String,
    },
}

pub(crate) struct CargoTestOutputParser {
    pub target: TestTarget,
}

impl CargoTestOutputParser {
    pub(crate) fn new(test_target: &TestTarget) -> Self {
        Self { target: test_target.clone() }
    }
}

impl JsonLinesParser<CargoTestMessage> for CargoTestOutputParser {
    fn from_line(&self, line: &str, _error: &mut String) -> Option<CargoTestMessage> {
        let mut deserializer = serde_json::Deserializer::from_str(line);
        deserializer.disable_recursion_limit();

        Some(CargoTestMessage {
            target: self.target.clone(),
            output: if let Ok(message) = CargoTestOutput::deserialize(&mut deserializer) {
                message
            } else {
                CargoTestOutput::Custom { text: line.to_owned() }
            },
        })
    }

    fn from_eof(&self) -> Option<CargoTestMessage> {
        Some(CargoTestMessage { target: self.target.clone(), output: CargoTestOutput::Finished })
    }
}

#[derive(Debug)]
pub(crate) struct CargoTestHandle {
    _handle: CommandHandle<CargoTestMessage>,
}

// Example of a cargo test command:
//
// cargo test --package my-package --bin my_bin --no-fail-fast -- module::func -Z unstable-options --format=json

#[derive(Debug, Clone)]
pub(crate) struct TestTarget {
    pub package: String,
    pub target: String,
    pub kind: TargetKind,
}

/// Configuration for the Test Explorer.
#[derive(Clone, Debug)]
pub(crate) struct CargoTestConfig {
    pub options: CargoOptions,
    pub runner: TestRunnerKind,
}

impl CargoTestHandle {
    pub(crate) fn new(
        path: Option<&str>,
        config: CargoTestConfig,
        root: &AbsPath,
        ws_target_dir: Option<&Utf8Path>,
        test_target: TestTarget,
        sender: Sender<CargoTestMessage>,
    ) -> anyhow::Result<Self> {
        let cmd = Self::automatic_command(
            path,
            config.options,
            config.runner,
            root,
            ws_target_dir,
            &test_target,
        );

        Ok(Self {
            _handle: CommandHandle::spawn(
                cmd,
                CargoTestOutputParser::new(&test_target),
                sender,
                None,
            )?,
        })
    }

    fn automatic_command(
        path: Option<&str>,
        options: CargoOptions,
        runner: TestRunnerKind,
        root: &AbsPath,
        ws_target_dir: Option<&Utf8Path>,
        test_target: &TestTarget,
    ) -> Command {
        match runner {
            TestRunnerKind::LibTest => {
                Self::libtest_command(path, options, root, ws_target_dir, test_target)
            }
            TestRunnerKind::Nextest => {
                Self::nextest_command(path, options, root, ws_target_dir, test_target)
            }
        }
    }

    /// Shared command builder for both libtest and nextest.
    /// Each caller is responsible for appending runner-specific trailing
    /// arguments (JSON format flags, filter expressions, etc.).
    fn cargo_base_command(
        subcommand: &[&str],
        options: &CargoOptions,
        root: &AbsPath,
        ws_target_dir: Option<&Utf8Path>,
        test_target: &TestTarget,
    ) -> Command {
        let mut cmd = toolchain::command(Tool::Cargo.path(), root, &options.extra_env);
        cmd.arg("--color=always");
        cmd.args(subcommand);

        cmd.arg("--package");
        cmd.arg(&test_target.package);

        if let TargetKind::Lib { .. } = test_target.kind {
            // no name required with lib because there can only be one lib target per package
            cmd.arg("--lib");
        } else if let Some(cargo_target) = test_target.kind.as_cargo_target() {
            cmd.arg(format!("--{cargo_target}"));
            cmd.arg(&test_target.target);
        } else {
            tracing::warn!("Running test for unknown cargo target {:?}", test_target.kind);
        }

        // --no-fail-fast is needed to ensure that all requested tests will run
        cmd.arg("--no-fail-fast");
        cmd.arg("--manifest-path");
        cmd.arg(root.join("Cargo.toml"));
        options.apply_on_command(&mut cmd, ws_target_dir);

        cmd
    }

    /// Build a `cargo test ... -- -Z unstable-options --format=json` command.
    fn libtest_command(
        path: Option<&str>,
        options: CargoOptions,
        root: &AbsPath,
        ws_target_dir: Option<&Utf8Path>,
        test_target: &TestTarget,
    ) -> Command {
        let mut cmd =
            Self::cargo_base_command(&["test"], &options, root, ws_target_dir, test_target);
        cmd.env("RUSTC_BOOTSTRAP", "1");
        cmd.arg("--");
        if let Some(path) = path {
            cmd.arg(path);
        }
        cmd.args(["-Z", "unstable-options"]);
        cmd.arg("--format=json");
        for extra_arg in options.extra_test_bin_args {
            cmd.arg(extra_arg);
        }
        cmd
    }

    /// Build a `cargo nextest run ... --message-format libtest-json` command.
    fn nextest_command(
        path: Option<&str>,
        options: CargoOptions,
        root: &AbsPath,
        ws_target_dir: Option<&Utf8Path>,
        test_target: &TestTarget,
    ) -> Command {
        let mut cmd = Self::cargo_base_command(
            &["nextest", "run"],
            &options,
            root,
            ws_target_dir,
            test_target,
        );
        cmd.arg("--message-format");
        cmd.arg("libtest-json");
        cmd.arg("--");
        if let Some(path) = path {
            cmd.arg(path);
        }
        cmd
    }
}
