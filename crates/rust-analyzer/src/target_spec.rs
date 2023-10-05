//! See `CargoTargetSpec`

use cfg::{CfgAtom, CfgExpr};
use ide::{Cancellable, CrateId, FileId, RunnableKind, TestId};
use project_model::{CargoFeatures, ManifestPath, TargetKind};
use rustc_hash::FxHashSet;
use vfs::{AbsPath, AbsPathBuf};

use crate::global_state::{GlobalStateSnapshot, TargetForCrateRoot};

/// Abstract representation of Cargo target.
///
/// We use it to cook up the set of cli args we need to pass to Cargo to
/// build/test/run the target.
#[derive(Clone)]
pub(crate) enum TargetSpec {
    Cargo(CargoTargetSpec),
    ProjectJson(ProjectJsonSpec),
}

impl TargetSpec {
    pub(crate) fn for_file(
        global_state_snapshot: &GlobalStateSnapshot,
        file_id: FileId,
    ) -> Cancellable<Option<TargetSpec>> {
        let crate_id = match &*global_state_snapshot.analysis.crates_for(file_id)? {
            &[crate_id, ..] => crate_id,
            _ => return Ok(None),
        };

        match global_state_snapshot.target_for_crate_root(crate_id) {
            Some(TargetForCrateRoot::Cargo(cargo_ws, target)) => {
                let target_data = &cargo_ws[target];
                let package_data = &cargo_ws[target_data.package];
                let res = CargoTargetSpec {
                    workspace_root: cargo_ws.workspace_root().to_path_buf(),
                    project_manifest: package_data.manifest.clone(),
                    crate_id,
                    package: cargo_ws.package_flag(package_data),
                    target: target_data.name.clone(),
                    target_kind: target_data.kind,
                    required_features: target_data.required_features.clone(),
                    features: package_data.features.keys().cloned().collect(),
                };
                Ok(Some(TargetSpec::Cargo(res)))
            }
            Some(TargetForCrateRoot::JsonProject(_, krate)) => match krate.target_spec {
                Some(target) => {
                    let manifest = ManifestPath::try_from(target.manifest_file).unwrap();
                    let res = ProjectJsonSpec {
                        workspace_root: manifest.parent().to_path_buf(),
                        project_manifest: manifest,
                        target_kind: target.target_kind,
                        target_label: target.target_label,
                        runnables: target.runnables,
                    };
                    Ok(Some(TargetSpec::ProjectJson(res)))
                }
                None => {
                    tracing::debug!(?krate, "no target spec");
                    Ok(None)
                }
            },
            None => {
                tracing::debug!(?crate_id, "no target found");
                Ok(None)
            }
        }
    }

    pub(crate) fn project_manifest(&self) -> &AbsPath {
        match self {
            TargetSpec::Cargo(cargo) => &cargo.project_manifest,
            TargetSpec::ProjectJson(project_json) => &project_json.project_manifest,
        }
    }

    pub(crate) fn workspace_root(&self) -> &AbsPath {
        match self {
            TargetSpec::Cargo(cargo) => &cargo.workspace_root,
            TargetSpec::ProjectJson(project_json) => &project_json.workspace_root,
        }
    }

    pub(crate) fn target_kind(&self) -> TargetKind {
        match self {
            TargetSpec::Cargo(cargo) => cargo.target_kind,
            TargetSpec::ProjectJson(project_json) => project_json.target_kind,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CargoTargetSpec {
    pub(crate) workspace_root: AbsPathBuf,
    /// [`project_manifest`] corresponds to the file defining a crate. With Cargo,
    /// this will be a Cargo.toml, but with a non-Cargo build system, this might be
    /// a `TARGETS` or `BUCK` file. Multiple, distinct crates can share a
    /// single `project_manifest`.
    pub(crate) project_manifest: ManifestPath,
    pub(crate) crate_id: CrateId,
    pub(crate) package: String,
    pub(crate) target: String,
    pub(crate) target_kind: TargetKind,
    pub(crate) required_features: Vec<String>,
    pub(crate) features: FxHashSet<String>,
}

#[derive(Clone)]
pub(crate) struct ProjectJsonSpec {
    pub(crate) workspace_root: AbsPathBuf,
    /// [`project_manifest`] corresponds to the file defining a crate. With Cargo,
    /// this will be a Cargo.toml, but with a non-Cargo build system, this might be
    /// a `TARGETS` or `BUCK` file. Multiple, distinct crates can share a
    /// single `project_manifest`.
    pub(crate) project_manifest: ManifestPath,
    pub(crate) target_label: String,
    pub(crate) target_kind: TargetKind,
    pub(crate) runnables: project_model::project_json::Runnables,
}

impl ProjectJsonSpec {
    pub(crate) fn runnable_args(spec: ProjectJsonSpec, kind: &RunnableKind) -> Vec<String> {
        let mut args = Vec::new();

        match kind {
            RunnableKind::Test { test_id, attr: _ } | RunnableKind::DocTest { test_id } => {
                let mut test_runnable = spec.runnables.test;
                for arg in &mut test_runnable.iter_mut() {
                    if arg == "{test_id}" {
                        *arg = test_id.to_string()
                    }
                }

                args.append(&mut test_runnable);
            }
            RunnableKind::TestMod { path } => {
                let mut test_runnable = spec.runnables.test;
                for arg in &mut test_runnable.iter_mut() {
                    if arg == "{test_id}" {
                        *arg = path.to_string()
                    }
                }

                args.append(&mut test_runnable);
            }
            RunnableKind::Bin => {
                let mut run = spec.runnables.run;
                args.append(&mut run);
            }
            _ => (),
        };

        args
    }
}

impl CargoTargetSpec {
    pub(crate) fn runnable_args(
        snap: &GlobalStateSnapshot,
        spec: CargoTargetSpec,
        kind: &RunnableKind,
        cfg: &Option<CfgExpr>,
    ) -> (Vec<String>, Vec<String>) {
        let mut args = Vec::new();
        let mut extra_args = Vec::new();

        match kind {
            RunnableKind::Test { test_id, attr } => {
                args.push("test".to_owned());
                extra_args.push(test_id.to_string());
                if let TestId::Path(_) = test_id {
                    extra_args.push("--exact".to_owned());
                }
                extra_args.push("--nocapture".to_owned());
                if attr.ignore {
                    extra_args.push("--ignored".to_owned());
                }
            }
            RunnableKind::TestMod { path } => {
                args.push("test".to_owned());
                extra_args.push(path.clone());
                extra_args.push("--nocapture".to_owned());
            }
            RunnableKind::Bench { test_id } => {
                args.push("bench".to_owned());
                extra_args.push(test_id.to_string());
                if let TestId::Path(_) = test_id {
                    extra_args.push("--exact".to_owned());
                }
                extra_args.push("--nocapture".to_owned());
            }
            RunnableKind::DocTest { test_id } => {
                args.push("test".to_owned());
                args.push("--doc".to_owned());
                extra_args.push(test_id.to_string());
                extra_args.push("--nocapture".to_owned());
            }
            RunnableKind::Bin => {
                let subcommand = match spec.target_kind {
                    TargetKind::Test => "test",
                    _ => "run",
                };
                args.push(subcommand.to_owned());
            }
        }

        let allowed_features = spec.features.clone();
        let target_required_features = spec.required_features.clone();

        spec.clone().push_to(&mut args, kind);

        let cargo_config = snap.config.cargo();

        match &cargo_config.features {
            CargoFeatures::All => {
                args.push("--all-features".to_owned());
                for feature in target_required_features {
                    args.push("--features".to_owned());
                    args.push(feature);
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
                    args.push("--features".to_owned());
                    args.push(feature);
                }

                if *no_default_features {
                    args.push("--no-default-features".to_owned());
                }
            }
        }
        (args, extra_args)
    }

    pub(crate) fn push_to(self, buf: &mut Vec<String>, kind: &RunnableKind) {
        let (package, target, target_kind) = (self.package, self.target, self.target_kind);

        buf.push("--package".to_owned());
        buf.push(package);

        // Can't mix --doc with other target flags
        if let RunnableKind::DocTest { .. } = kind {
            return;
        }
        match target_kind {
            TargetKind::Bin => {
                buf.push("--bin".to_owned());
                buf.push(target);
            }
            TargetKind::Test => {
                buf.push("--test".to_owned());
                buf.push(target);
            }
            TargetKind::Bench => {
                buf.push("--bench".to_owned());
                buf.push(target);
            }
            TargetKind::Example => {
                buf.push("--example".to_owned());
                buf.push(target);
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
        CfgExpr::Atom(CfgAtom::KeyValue { key, value }) if key == "feature" => {
            features.push(value.to_string())
        }
        CfgExpr::All(preds) => {
            preds.iter().for_each(|cfg| required_features(cfg, features));
        }
        CfgExpr::Any(preds) => {
            for cfg in preds {
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

    use mbe::{syntax_node_to_token_tree, DummyTestSpanMap, DUMMY};
    use syntax::{
        ast::{self, AstNode},
        SmolStr,
    };

    fn check(cfg: &str, expected_features: &[&str]) {
        let cfg_expr = {
            let source_file = ast::SourceFile::parse(cfg).ok().unwrap();
            let tt = source_file.syntax().descendants().find_map(ast::TokenTree::cast).unwrap();
            let tt = syntax_node_to_token_tree(tt.syntax(), &DummyTestSpanMap, DUMMY);
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
}
