//! See `TargetSpec`

use std::mem;

use cargo_metadata::PackageId;
use cfg::{CfgAtom, CfgExpr};
use hir::sym;
use ide::{Cancellable, Crate, FileId, RunnableKind, TestId};
use project_model::project_json::{self, Runnable};
use project_model::{CargoFeatures, ManifestPath, TargetKind};
use rustc_hash::FxHashSet;
use triomphe::Arc;
use vfs::AbsPathBuf;

use crate::config::TestRunnerKind;
use crate::global_state::GlobalStateSnapshot;

/// A target represents a thing we can build or test.
///
/// We use it to calculate the CLI arguments required to build, run or
/// test the target.
#[derive(Clone, Debug)]
pub(crate) enum TargetSpec {
    Cargo(CargoTargetSpec),
    ProjectJson(ProjectJsonTargetSpec),
}

impl TargetSpec {
    pub(crate) fn for_file(
        global_state_snapshot: &GlobalStateSnapshot,
        file_id: FileId,
    ) -> Cancellable<Option<Self>> {
        let crate_id = match &*global_state_snapshot.analysis.crates_for(file_id)? {
            &[crate_id, ..] => crate_id,
            _ => return Ok(None),
        };

        Ok(global_state_snapshot.target_spec_for_crate(crate_id))
    }

    pub(crate) fn target_kind(&self) -> TargetKind {
        match self {
            TargetSpec::Cargo(cargo) => cargo.target_kind,
            TargetSpec::ProjectJson(project_json) => project_json.target_kind,
        }
    }
}

/// Abstract representation of Cargo target.
///
/// We use it to cook up the set of cli args we need to pass to Cargo to
/// build/test/run the target.
#[derive(Clone, Debug)]
pub(crate) struct CargoTargetSpec {
    pub(crate) workspace_root: AbsPathBuf,
    pub(crate) cargo_toml: ManifestPath,
    pub(crate) package: String,
    pub(crate) package_id: Arc<PackageId>,
    pub(crate) target: String,
    pub(crate) target_kind: TargetKind,
    pub(crate) crate_id: Crate,
    pub(crate) required_features: Vec<String>,
    pub(crate) features: FxHashSet<String>,
    pub(crate) sysroot_root: Option<vfs::AbsPathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectJsonTargetSpec {
    pub(crate) label: String,
    pub(crate) target_kind: TargetKind,
    pub(crate) shell_runnables: Vec<Runnable>,
    pub(crate) project_root: AbsPathBuf,
}

impl ProjectJsonTargetSpec {
    fn find_replace_runnable(
        &self,
        kind: project_json::RunnableKind,
        replacer: &dyn Fn(&Self, &str) -> String,
    ) -> Option<Runnable> {
        for runnable in &self.shell_runnables {
            if runnable.kind == kind {
                let mut runnable = runnable.clone();

                let replaced_args: Vec<_> =
                    runnable.args.iter().map(|arg| replacer(self, arg)).collect();
                runnable.args = replaced_args;

                return Some(runnable);
            }
        }

        None
    }

    pub(crate) fn runnable_args(&self, kind: &RunnableKind) -> Option<Runnable> {
        match kind {
            RunnableKind::Bin => self
                .find_replace_runnable(project_json::RunnableKind::Run, &|this, arg| {
                    arg.replace("{label}", &this.label)
                }),
            RunnableKind::Test { test_id, .. } => {
                self.find_replace_runnable(project_json::RunnableKind::Run, &|this, arg| {
                    arg.replace("{label}", &this.label).replace("{test_id}", &test_id.to_string())
                })
            }
            RunnableKind::TestMod { path } => self
                .find_replace_runnable(project_json::RunnableKind::TestMod, &|this, arg| {
                    arg.replace("{label}", &this.label).replace("{test_pattern}", path)
                }),
            RunnableKind::Bench { test_id } => {
                self.find_replace_runnable(project_json::RunnableKind::BenchOne, &|this, arg| {
                    arg.replace("{label}", &this.label).replace("{bench_id}", &test_id.to_string())
                })
            }
            RunnableKind::DocTest { test_id } => {
                self.find_replace_runnable(project_json::RunnableKind::DocTestOne, &|this, arg| {
                    arg.replace("{label}", &this.label).replace("{test_id}", &test_id.to_string())
                })
            }
        }
    }
}

impl CargoTargetSpec {
    pub(crate) fn runnable_args(
        snap: &GlobalStateSnapshot,
        spec: Option<CargoTargetSpec>,
        kind: &RunnableKind,
        cfg: &Option<CfgExpr>,
    ) -> (Vec<String>, Vec<String>) {
        let config = snap.config.runnables(None);
        let extra_test_binary_args = config.extra_test_binary_args;

        let mut cargo_args = Vec::new();
        let executable_args =
            Self::executable_args_for(kind, extra_test_binary_args, config.runner);

        // For LibTest the user-configurable `test_command` / `bench_command`
        // is used (defaults: "test", "bench").  For Nextest the subcommand is
        // hard-coded to "nextest run" — `test_command` is intentionally
        // ignored because it has no meaningful nextest equivalent.
        match kind {
            RunnableKind::Test { .. } | RunnableKind::TestMod { .. } => match config.runner {
                TestRunnerKind::LibTest => cargo_args.push(config.test_command),
                TestRunnerKind::Nextest => {
                    cargo_args.push("nextest".to_owned());
                    cargo_args.push("run".to_owned());
                }
            },
            RunnableKind::Bench { .. } => {
                // nextest has no bench subcommand; this always uses the
                // libtest bench_command regardless of the runner setting.
                cargo_args.push(config.bench_command);
            }
            RunnableKind::DocTest { .. } => {
                cargo_args.push("test".to_owned());
                cargo_args.push("--doc".to_owned());
            }
            RunnableKind::Bin => {
                let is_test_target =
                    matches!(spec, Some(CargoTargetSpec { target_kind: TargetKind::Test, .. }));
                match (is_test_target, config.runner) {
                    (true, TestRunnerKind::Nextest) => {
                        cargo_args.push("nextest".to_owned());
                        cargo_args.push("run".to_owned());
                    }
                    (true, TestRunnerKind::LibTest) => {
                        cargo_args.push(config.test_command);
                    }
                    _ => cargo_args.push("run".to_owned()),
                }
            }
        }

        let (allowed_features, target_required_features) = if let Some(mut spec) = spec {
            let allowed_features = mem::take(&mut spec.features);
            let required_features = mem::take(&mut spec.required_features);
            spec.push_to(&mut cargo_args, kind);
            (allowed_features, required_features)
        } else {
            (Default::default(), Default::default())
        };

        let cargo_config = snap.config.cargo(None);

        match &cargo_config.features {
            CargoFeatures::All => {
                cargo_args.push("--all-features".to_owned());
                for feature in target_required_features {
                    cargo_args.push("--features".to_owned());
                    cargo_args.push(feature);
                }
            }
            CargoFeatures::Selected { features, no_default_features } => {
                let mut feats = Vec::new();
                if let Some(cfg) = cfg.as_ref() {
                    required_features(cfg, &mut feats);
                }

                feats.extend(
                    features.iter().filter(|&feat| allowed_features.contains(feat)).cloned(),
                );
                feats.extend(target_required_features);

                feats.dedup();
                for feature in feats {
                    cargo_args.push("--features".to_owned());
                    cargo_args.push(feature);
                }

                if *no_default_features {
                    cargo_args.push("--no-default-features".to_owned());
                }
            }
        }
        cargo_args.extend(config.cargo_extra_args.iter().cloned());
        (cargo_args, executable_args)
    }

    pub(crate) fn override_command(
        snap: &GlobalStateSnapshot,
        spec: Option<CargoTargetSpec>,
        kind: &RunnableKind,
    ) -> Option<Vec<String>> {
        let config = snap.config.runnables(None);
        let (args, test_name) = match kind {
            RunnableKind::Test { test_id, .. } => {
                (config.test_override_command, Some(test_id.to_string()))
            }
            RunnableKind::TestMod { path } => (config.test_override_command, Some(path.clone())),
            RunnableKind::Bench { test_id } => {
                (config.bench_override_command, Some(test_id.to_string()))
            }
            RunnableKind::DocTest { test_id } => {
                (config.doc_test_override_command, Some(test_id.to_string()))
            }
            RunnableKind::Bin => match spec {
                Some(CargoTargetSpec { target_kind: TargetKind::Test, .. }) => {
                    (config.test_override_command, None)
                }
                _ => (None, None),
            },
        };
        let test_name = test_name.unwrap_or_default();

        let target_arg = |kind| match kind {
            TargetKind::Bin => "--bin",
            TargetKind::Test => "--test",
            TargetKind::Bench => "--bench",
            TargetKind::Example => "--example",
            TargetKind::Lib { .. } => "--lib",
            TargetKind::BuildScript | TargetKind::Other => "",
        };

        let target = |kind, target| match kind {
            TargetKind::Bin | TargetKind::Test | TargetKind::Bench | TargetKind::Example => target,
            _ => "",
        };

        let replace_placeholders = |arg: String| match &spec {
            Some(spec) => arg
                .replace("${package}", &spec.package)
                .replace("${target_arg}", target_arg(spec.target_kind))
                .replace("${target}", target(spec.target_kind, &spec.target))
                .replace("${test_name}", &test_name),
            _ => arg,
        };

        let executable_args =
            Self::executable_args_for(kind, config.extra_test_binary_args, config.runner);

        args.map(|mut args| {
            if let Some(idx) = args.iter().position(|a| a == "${executable_args}") {
                args.splice(idx..idx + 1, executable_args);
            }

            args.into_iter().map(replace_placeholders).filter(|a| !a.trim().is_empty()).collect()
        })
    }

    fn executable_args_for(
        kind: &RunnableKind,
        extra_test_binary_args: impl IntoIterator<Item = String>,
        runner: TestRunnerKind,
    ) -> Vec<String> {
        let mut executable_args = Vec::new();
        match runner {
            TestRunnerKind::LibTest => match kind {
                RunnableKind::Test { test_id } => {
                    executable_args.push(test_id.to_string());
                    if let TestId::Path(_) = test_id {
                        executable_args.push("--exact".to_owned());
                    }
                    executable_args.extend(extra_test_binary_args);
                    executable_args.push("--include-ignored".to_owned());
                }
                RunnableKind::TestMod { path } => {
                    executable_args.push(path.clone());
                    executable_args.extend(extra_test_binary_args);
                }
                RunnableKind::Bench { test_id } => {
                    executable_args.push(test_id.to_string());
                    if let TestId::Path(_) = test_id {
                        executable_args.push("--exact".to_owned());
                    }
                    executable_args.extend(extra_test_binary_args);
                }
                RunnableKind::DocTest { test_id } => {
                    executable_args.push(test_id.to_string());
                    executable_args.extend(extra_test_binary_args);
                }
                RunnableKind::Bin => {}
            },
            TestRunnerKind::Nextest => match kind {
                RunnableKind::Test { test_id } => {
                    executable_args.push(test_id.to_string());
                }
                RunnableKind::TestMod { path } => {
                    executable_args.push(path.clone());
                }
                RunnableKind::Bench { test_id } => {
                    executable_args.push(test_id.to_string());
                }
                RunnableKind::DocTest { test_id } => {
                    executable_args.push(test_id.to_string());
                }
                RunnableKind::Bin => {}
            },
        }

        executable_args
    }

    pub(crate) fn push_to(self, buf: &mut Vec<String>, kind: &RunnableKind) {
        buf.push("--package".to_owned());
        buf.push(self.package);

        // Can't mix --doc with other target flags
        if let RunnableKind::DocTest { .. } = kind {
            return;
        }
        match self.target_kind {
            TargetKind::Bin => {
                buf.push("--bin".to_owned());
                buf.push(self.target);
            }
            TargetKind::Test => {
                buf.push("--test".to_owned());
                buf.push(self.target);
            }
            TargetKind::Bench => {
                buf.push("--bench".to_owned());
                buf.push(self.target);
            }
            TargetKind::Example => {
                buf.push("--example".to_owned());
                buf.push(self.target);
            }
            TargetKind::Lib { is_proc_macro: _ } => {
                buf.push("--lib".to_owned());
            }
            TargetKind::Other | TargetKind::BuildScript => (),
        }
    }
}

/// Fill minimal features needed
fn required_features(cfg_expr: &CfgExpr, features: &mut Vec<String>) {
    match cfg_expr {
        CfgExpr::Atom(CfgAtom::KeyValue { key, value }) if *key == sym::feature => {
            features.push(value.to_string())
        }
        CfgExpr::All(preds) => {
            preds.iter().for_each(|cfg| required_features(cfg, features));
        }
        CfgExpr::Any(preds) => {
            for cfg in preds.iter() {
                let len_features = features.len();
                required_features(cfg, features);
                if len_features != features.len() {
                    break;
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ide::Edition;
    use syntax::{
        SmolStr,
        ast::{self, AstNode},
    };
    use syntax_bridge::{
        DocCommentDesugarMode,
        dummy_test_span_utils::{DUMMY, DummyTestSpanMap},
        syntax_node_to_token_tree,
    };

    fn check(cfg: &str, expected_features: &[&str]) {
        let cfg_expr = {
            let source_file = ast::SourceFile::parse(cfg, Edition::CURRENT).ok().unwrap();
            let tt = source_file.syntax().descendants().find_map(ast::TokenTree::cast).unwrap();
            let tt = syntax_node_to_token_tree(
                tt.syntax(),
                &DummyTestSpanMap,
                DUMMY,
                DocCommentDesugarMode::Mbe,
            );
            CfgExpr::parse(&tt)
        };

        let mut features = vec![];
        required_features(&cfg_expr, &mut features);

        let expected_features =
            expected_features.iter().map(|&it| SmolStr::new(it)).collect::<Vec<_>>();

        assert_eq!(features, expected_features);
    }

    #[test]
    fn test_cfg_expr_minimal_features_needed() {
        check(r#"#![cfg(feature = "baz")]"#, &["baz"]);
        check(r#"#![cfg(all(feature = "baz", feature = "foo"))]"#, &["baz", "foo"]);
        check(r#"#![cfg(any(feature = "baz", feature = "foo", unix))]"#, &["baz"]);
        check(r#"#![cfg(foo)]"#, &[]);
    }

    #[test]
    fn executable_args_libtest_path_test() {
        let kind = RunnableKind::Test { test_id: TestId::Path("mod::my_test".into()) };
        let args = CargoTargetSpec::executable_args_for(
            &kind,
            vec!["--nocapture".to_owned()],
            TestRunnerKind::LibTest,
        );
        assert_eq!(args, vec!["mod::my_test", "--exact", "--nocapture", "--include-ignored"]);
    }

    #[test]
    fn executable_args_libtest_name_test() {
        let kind = RunnableKind::Test { test_id: TestId::Name("my_test".into()) };
        let args = CargoTargetSpec::executable_args_for(
            &kind,
            vec!["--nocapture".to_owned()],
            TestRunnerKind::LibTest,
        );
        // Name tests don't get --exact
        assert_eq!(args, vec!["my_test", "--nocapture", "--include-ignored"]);
    }

    #[test]
    fn executable_args_nextest_path_test() {
        let kind = RunnableKind::Test { test_id: TestId::Path("mod::my_test".into()) };
        let args = CargoTargetSpec::executable_args_for(
            &kind,
            vec!["--nocapture".to_owned()],
            TestRunnerKind::Nextest,
        );
        // Nextest: no --exact, no --include-ignored, no extra binary args
        assert_eq!(args, vec!["mod::my_test"]);
    }

    #[test]
    fn executable_args_nextest_test_mod() {
        let kind = RunnableKind::TestMod { path: "my_module".into() };
        let args = CargoTargetSpec::executable_args_for(
            &kind,
            vec!["--nocapture".to_owned()],
            TestRunnerKind::Nextest,
        );
        // Nextest: only the path, no extra binary args
        assert_eq!(args, vec!["my_module"]);
    }

    #[test]
    fn executable_args_libtest_bench() {
        let kind = RunnableKind::Bench { test_id: TestId::Path("mod::my_bench".into()) };
        let args = CargoTargetSpec::executable_args_for(
            &kind,
            vec!["--nocapture".to_owned()],
            TestRunnerKind::LibTest,
        );
        assert_eq!(args, vec!["mod::my_bench", "--exact", "--nocapture"]);
    }

    #[test]
    fn executable_args_nextest_bench() {
        let kind = RunnableKind::Bench { test_id: TestId::Path("mod::my_bench".into()) };
        let args = CargoTargetSpec::executable_args_for(
            &kind,
            vec!["--nocapture".to_owned()],
            TestRunnerKind::Nextest,
        );
        // Nextest: no --exact, no extra args
        assert_eq!(args, vec!["mod::my_bench"]);
    }
}
