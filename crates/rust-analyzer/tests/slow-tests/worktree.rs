use rust_analyzer::lsp::ext::{ViewCrateGraphParams, ViewCrateGraphRequest};
use test_utils::skip_slow_tests;

use crate::support::Project;

/// A checkout with two packages, and a git worktree of it at `/wt` in which `$APP` is the
/// contents of `app/src/lib.rs`.
const CHECKOUT_AND_WORKTREE: &str = r#"
//- /base/Cargo.toml
[workspace]
members = ["core_lib", "app"]
resolver = "2"

//- /base/core_lib/Cargo.toml
[package]
name = "core_lib"
version = "0.0.0"

//- /base/core_lib/src/lib.rs
pub fn answer() -> u32 { 42 }

//- /base/app/Cargo.toml
[package]
name = "app"
version = "0.0.0"

[dependencies]
core_lib = { path = "../core_lib" }

//- /base/app/src/lib.rs
pub fn run() -> u32 { core_lib::answer() }

//- /wt/.git
gitdir: ../base/.git/worktrees/wt

//- /wt/Cargo.toml
[workspace]
members = ["core_lib", "app"]
resolver = "2"

//- /wt/core_lib/Cargo.toml
[package]
name = "core_lib"
version = "0.0.0"

//- /wt/core_lib/src/lib.rs
pub fn answer() -> u32 { 42 }

//- /wt/app/Cargo.toml
[package]
name = "app"
version = "0.0.0"

[dependencies]
core_lib = { path = "../core_lib" }

//- /wt/app/src/lib.rs
$APP

"#;

fn crate_count(crate_graph: &str, name: &str) -> usize {
    crate_graph.matches(&format!("label=\"{name}\"")).count()
}

#[test]
fn worktree_shares_unchanged_crates_with_its_base_checkout() {
    if skip_slow_tests() {
        return;
    }

    let fixture =
        CHECKOUT_AND_WORKTREE.replace("$APP", "pub fn run() -> u32 { core_lib::answer() + 1 }");
    let server = Project::with_fixture(&fixture)
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    let crate_graph =
        server.send_request::<ViewCrateGraphRequest>(ViewCrateGraphParams { full: true });
    let crate_graph = crate_graph.as_str().unwrap();
    // `core_lib` is the same in both, `app` differs
    assert_eq!(crate_count(crate_graph, "core_lib"), 1, "{crate_graph}");
    assert_eq!(crate_count(crate_graph, "app"), 2, "{crate_graph}");
}

#[test]
fn worktree_identical_to_its_base_checkout_adds_no_crates() {
    if skip_slow_tests() {
        return;
    }

    let fixture =
        CHECKOUT_AND_WORKTREE.replace("$APP", "pub fn run() -> u32 { core_lib::answer() }");
    let server = Project::with_fixture(&fixture)
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    let crate_graph =
        server.send_request::<ViewCrateGraphRequest>(ViewCrateGraphParams { full: true });
    let crate_graph = crate_graph.as_str().unwrap();
    assert_eq!(crate_count(crate_graph, "core_lib"), 1, "{crate_graph}");
    assert_eq!(crate_count(crate_graph, "app"), 1, "{crate_graph}");
}

#[test]
fn worktree_with_changed_dependency_shares_nothing_that_depends_on_it() {
    if skip_slow_tests() {
        return;
    }

    let fixture = CHECKOUT_AND_WORKTREE
        .replace("$APP", "pub fn run() -> u32 { core_lib::answer() }")
        .replacen(
            "//- /wt/core_lib/src/lib.rs\npub fn answer() -> u32 { 42 }",
            "//- /wt/core_lib/src/lib.rs\npub fn answer() -> u32 { 43 }",
            1,
        );
    let server = Project::with_fixture(&fixture)
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    let crate_graph =
        server.send_request::<ViewCrateGraphRequest>(ViewCrateGraphParams { full: true });
    let crate_graph = crate_graph.as_str().unwrap();
    // `app` is unchanged, but it depends on the changed `core_lib`
    assert_eq!(crate_count(crate_graph, "core_lib"), 2, "{crate_graph}");
    assert_eq!(crate_count(crate_graph, "app"), 2, "{crate_graph}");
}
