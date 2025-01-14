use std::{fmt, process::Command};

use ide_db::FxHashMap;
use paths::Utf8PathBuf;
use toolchain::Tool;
use vfs::{AbsPath, AbsPathBuf};

pub(crate) const SAVED_FILE_PLACEHOLDER: &str = "$saved_file";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum InvocationStrategy {
    Once,
    #[default]
    PerWorkspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CargoOptions {
    pub(crate) target_tuples: Vec<String>,
    pub(crate) all_targets: bool,
    pub(crate) no_default_features: bool,
    pub(crate) all_features: bool,
    pub(crate) features: Vec<String>,
    pub(crate) extra_args: Vec<String>,
    pub(crate) extra_test_bin_args: Vec<String>,
    pub(crate) extra_env: FxHashMap<String, String>,
    pub(crate) target_dir: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) enum Target {
    Bin(String),
    Example(String),
    Benchmark(String),
    Test(String),
}

impl CargoOptions {
    pub(crate) fn apply_on_command(&self, cmd: &mut Command) {
        for target in &self.target_tuples {
            cmd.args(["--target", target.as_str()]);
        }
        if self.all_targets {
            cmd.arg("--all-targets");
        }
        if self.all_features {
            cmd.arg("--all-features");
        } else {
            if self.no_default_features {
                cmd.arg("--no-default-features");
            }
            if !self.features.is_empty() {
                cmd.arg("--features");
                cmd.arg(self.features.join(" "));
            }
        }
        if let Some(target_dir) = &self.target_dir {
            cmd.arg("--target-dir").arg(target_dir);
        }
        cmd.envs(&self.extra_env);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FlycheckConfig {
    CargoCommand {
        command: String,
        options: CargoOptions,
        ansi_color_output: bool,
    },
    CustomCommand {
        command: String,
        args: Vec<String>,
        extra_env: FxHashMap<String, String>,
        invocation_strategy: InvocationStrategy,
    },
}

impl fmt::Display for FlycheckConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlycheckConfig::CargoCommand { command, .. } => write!(f, "cargo {command}"),
            FlycheckConfig::CustomCommand { command, args, .. } => {
                write!(f, "{command} {}", args.join(" "))
            }
        }
    }
}

/// Construct a `Command` object for checking the user's code. If the user
/// has specified a custom command with placeholders that we cannot fill,
/// return None.
pub(super) fn check_command(
    root: &AbsPathBuf,
    sysroot_root: &Option<AbsPathBuf>,
    manifest_path: &Option<AbsPathBuf>,
    config: FlycheckConfig,
    package: Option<&str>,
    saved_file: Option<&AbsPath>,
    target: Option<Target>,
) -> Option<Command> {
    match config {
        FlycheckConfig::CargoCommand { command, options, ansi_color_output } => {
            let mut cmd = toolchain::command(Tool::Cargo.path(), &*root);
            if let Some(sysroot_root) = &sysroot_root {
                cmd.env("RUSTUP_TOOLCHAIN", AsRef::<std::path::Path>::as_ref(sysroot_root));
            }
            cmd.arg(command);

            match package {
                Some(pkg) => cmd.arg("-p").arg(pkg),
                None => cmd.arg("--workspace"),
            };

            if let Some(tgt) = target {
                match tgt {
                    Target::Bin(tgt) => cmd.arg("--bin").arg(tgt),
                    Target::Example(tgt) => cmd.arg("--example").arg(tgt),
                    Target::Test(tgt) => cmd.arg("--test").arg(tgt),
                    Target::Benchmark(tgt) => cmd.arg("--bench").arg(tgt),
                };
            }

            cmd.arg(if ansi_color_output {
                "--message-format=json-diagnostic-rendered-ansi"
            } else {
                "--message-format=json"
            });

            if let Some(manifest_path) = &manifest_path {
                cmd.arg("--manifest-path");
                cmd.arg(manifest_path);
                if manifest_path.extension() == Some("rs") {
                    cmd.arg("-Zscript");
                }
            }

            cmd.arg("--keep-going");

            options.apply_on_command(&mut cmd);
            cmd.args(&options.extra_args);
            Some(cmd)
        }
        FlycheckConfig::CustomCommand { command, args, extra_env, invocation_strategy } => {
            let root = match invocation_strategy {
                InvocationStrategy::Once => &*root,
                InvocationStrategy::PerWorkspace => {
                    // FIXME: &affected_workspace
                    &*root
                }
            };
            let mut cmd = toolchain::command(command, root);
            cmd.envs(extra_env);

            // If the custom command has a $saved_file placeholder, and
            // we're saving a file, replace the placeholder in the arguments.
            if let Some(saved_file) = saved_file {
                for arg in args {
                    if arg == SAVED_FILE_PLACEHOLDER {
                        cmd.arg(saved_file);
                    } else {
                        cmd.arg(arg);
                    }
                }
            } else {
                for arg in args {
                    if arg == SAVED_FILE_PLACEHOLDER {
                        // The custom command has a $saved_file placeholder,
                        // but we had an IDE event that wasn't a file save. Do nothing.
                        return None;
                    }

                    cmd.arg(arg);
                }
            }

            Some(cmd)
        }
    }
}
