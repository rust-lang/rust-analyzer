use lsp_types::{
    DefinitionParams, DefinitionRequest, DidChangeTextDocumentNotification,
    DidChangeTextDocumentParams, DidOpenTextDocumentNotification, DidOpenTextDocumentParams,
    HoverParams, HoverRequest, LanguageKind, Position, TextDocumentContentChangeEvent,
    TextDocumentContentChangeWholeDocument, TextDocumentItem, TextDocumentPositionParams,
    VersionedTextDocumentIdentifier,
};
use rust_analyzer::lsp::ext::{ViewCrateGraphParams, ViewCrateGraphRequest};
use test_utils::skip_slow_tests;

use crate::support::{Project, Server};

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

fn share_worktrees() -> serde_json::Value {
    serde_json::json!({ "workspace": { "shareWorktrees": true } })
}

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
        .with_config(share_worktrees())
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
        .with_config(share_worktrees())
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
        .with_config(share_worktrees())
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

fn worktree_server(app: &str) -> Server {
    let fixture = CHECKOUT_AND_WORKTREE.replace("$APP", app);
    Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded()
}

fn position(server: &Server, path: &str, line: u32, character: u32) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: server.doc_id(path),
        position: Position { line, character },
    }
}

fn wait_for_crate_count(server: &Server, name: &str, count: usize) {
    let mut attempts = 0;
    loop {
        let crate_graph =
            server.send_request::<ViewCrateGraphRequest>(ViewCrateGraphParams { full: true });
        let crate_graph = crate_graph.as_str().unwrap();
        if crate_count(crate_graph, name) == count {
            break;
        }
        attempts += 1;
        assert!(attempts < 500, "expected {count} `{name}` crates: {crate_graph}");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn worktree_file_of_shared_crate_is_served_under_its_own_path() {
    if skip_slow_tests() {
        return;
    }

    let server = worktree_server("pub fn run() -> u32 { core_lib::answer() + 1 }");

    // `core_lib` of the worktree is not analyzed on its own, yet its files answer requests
    let hover = server.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: position(&server, "wt/core_lib/src/lib.rs", 0, 8),
        work_done_progress_params: Default::default(),
    });
    assert!(hover.to_string().contains("pub fn answer() -> u32"), "{hover}");

    // Going from the worktree's own crate into the shared one stays in the worktree
    let definition = server.send_request::<DefinitionRequest>(DefinitionParams {
        text_document_position_params: position(&server, "wt/app/src/lib.rs", 0, 34),
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    });
    let definition = definition.to_string();
    assert!(definition.contains("/wt/core_lib/src/lib.rs"), "{definition}");
    assert!(!definition.contains("/base/"), "{definition}");

    // While the base checkout is answered with its own paths
    let definition = server.send_request::<DefinitionRequest>(DefinitionParams {
        text_document_position_params: position(&server, "base/app/src/lib.rs", 0, 34),
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    });
    let definition = definition.to_string();
    assert!(definition.contains("/base/core_lib/src/lib.rs"), "{definition}");
    assert!(!definition.contains("/wt/"), "{definition}");
}

#[test]
fn editing_a_shared_crate_in_the_worktree_stops_sharing_it() {
    if skip_slow_tests() {
        return;
    }

    let server = worktree_server("pub fn run() -> u32 { core_lib::answer() }");
    wait_for_crate_count(&server, "core_lib", 1);
    wait_for_crate_count(&server, "app", 1);

    let doc_id = server.doc_id("wt/core_lib/src/lib.rs");
    let text_on_disk = std::fs::read_to_string(doc_id.uri.to_file_path().unwrap()).unwrap();
    server.notification::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: text_on_disk.clone(),
        },
    });
    let change = |version: i32, text: &str| DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            text_document_identifier: doc_id.clone(),
            version,
        },
        content_changes: vec![
            TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                TextDocumentContentChangeWholeDocument { text: text.to_owned() },
            ),
        ],
    };

    // The worktree gets its own `core_lib`, and its own `app` as that depends on it
    server.notification::<DidChangeTextDocumentNotification>(change(
        2,
        "pub fn answer() -> u64 { 42 }\n",
    ));
    wait_for_crate_count(&server, "core_lib", 2);
    wait_for_crate_count(&server, "app", 2);
    let hover = |path: &str| {
        server
            .send_request::<HoverRequest>(HoverParams {
                text_document_position_params: position(&server, path, 0, 8),
                work_done_progress_params: Default::default(),
            })
            .to_string()
    };
    assert!(hover("wt/core_lib/src/lib.rs").contains("pub fn answer() -> u64"));
    assert!(hover("base/core_lib/src/lib.rs").contains("pub fn answer() -> u32"));

    // Undoing the edit shares them again
    server.notification::<DidChangeTextDocumentNotification>(change(3, &text_on_disk));
    wait_for_crate_count(&server, "core_lib", 1);
    wait_for_crate_count(&server, "app", 1);
    assert!(hover("wt/core_lib/src/lib.rs").contains("pub fn answer() -> u32"));
}

#[test]
fn references_from_a_worktree_show_the_worktree_not_the_base_checkout() {
    if skip_slow_tests() {
        return;
    }

    let server = worktree_server("pub fn run() -> u32 { core_lib::answer() + 1 }");
    let references = |path: &str| {
        server
            .send_request::<lsp_types::ReferencesRequest>(lsp_types::ReferenceParams {
                text_document_position_params: position(&server, path, 0, 8),
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                context: lsp_types::ReferenceContext { include_declaration: false },
            })
            .to_string()
    };

    // `core_lib` is shared, but the worktree has its own `app` that uses it
    let from_worktree = references("wt/core_lib/src/lib.rs");
    assert!(from_worktree.contains("/wt/app/src/lib.rs"), "{from_worktree}");
    assert!(!from_worktree.contains("/base/"), "{from_worktree}");

    let from_base = references("base/core_lib/src/lib.rs");
    assert!(from_base.contains("/base/app/src/lib.rs"), "{from_base}");
}

#[test]
fn client_working_in_a_worktree_sees_symbols_of_the_worktree() {
    if skip_slow_tests() {
        return;
    }

    let fixture =
        CHECKOUT_AND_WORKTREE.replace("$APP", "pub fn run() -> u32 { core_lib::answer() + 1 }");
    let multi_server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .multi_server();
    let base_client = multi_server.connect(rust_analyzer::ClientId(1));
    base_client.wait_until_workspace_is_loaded();

    let worktree_client = multi_server.connect_with_encoding(rust_analyzer::ClientId(2), None);
    let worktree_root = worktree_client.doc_id("wt").uri;
    let params: lsp_types::InitializeParams = serde_json::from_value(serde_json::json!({
        "rootUri": worktree_root,
        "capabilities": {},
    }))
    .unwrap();
    worktree_client.send_request::<lsp_types::InitializeRequest>(params);
    worktree_client
        .notification::<lsp_types::InitializedNotification>(lsp_types::InitializedParams {});

    let symbols = |query: &str| {
        worktree_client
            .send_request::<rust_analyzer::lsp::ext::WorkspaceSymbolRequest>(
                rust_analyzer::lsp::ext::WorkspaceSymbolParams {
                    partial_result_params: Default::default(),
                    work_done_progress_params: Default::default(),
                    query: query.to_owned(),
                    search_scope: None,
                    search_kind: None,
                },
            )
            .to_string()
    };

    // The worktree's own crate, and not the base checkout's version of it
    let run = symbols("run#");
    assert!(run.contains("/wt/app/src/lib.rs"), "{run}");
    assert!(!run.contains("/base/"), "{run}");

    // A shared crate, under the worktree's path
    let answer = symbols("answer#");
    assert!(answer.contains("/wt/core_lib/src/lib.rs"), "{answer}");
    assert!(!answer.contains("/base/"), "{answer}");

    base_client.shutdown_and_exit();
    worktree_client.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn opening_only_the_worktree_loads_its_base_checkout() {
    if skip_slow_tests() {
        return;
    }

    let fixture =
        CHECKOUT_AND_WORKTREE.replace("$APP", "pub fn run() -> u32 { core_lib::answer() + 1 }");
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    wait_for_crate_count(&server, "core_lib", 1);
    wait_for_crate_count(&server, "app", 2);
    let hover = server.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: position(&server, "wt/core_lib/src/lib.rs", 0, 8),
        work_done_progress_params: Default::default(),
    });
    assert!(hover.to_string().contains("pub fn answer() -> u32"), "{hover}");
}

#[test]
fn worktrees_are_not_shared_unless_asked_for() {
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

    wait_for_crate_count(&server, "core_lib", 2);
    wait_for_crate_count(&server, "app", 2);
}

/// The fixture with `app` pulling in `shared.rs` from the root of its checkout.
fn pulling_in_fixture(base_shared: &str, worktree_shared: &str) -> String {
    let app =
        "#[path = \"../../shared.rs\"]\nmod shared;\npub fn run() -> u32 { core_lib::answer() }";
    CHECKOUT_AND_WORKTREE.replace("$APP", app).replace(
        "pub fn run() -> u32 { core_lib::answer() }\n\n//- /wt/.git",
        &format!("{app}\n\n//- /wt/.git"),
    ) + &format!("//- /base/shared.rs\n{base_shared}\n\n//- /wt/shared.rs\n{worktree_shared}\n\n")
}

#[test]
fn crate_pulling_in_a_file_that_differs_is_not_shared() {
    if skip_slow_tests() {
        return;
    }

    // `app` itself is identical in both, but what it pulls in from outside of its package is not
    let fixture = pulling_in_fixture("pub fn shared() {}", "pub fn shared() -> u8 { 1 }");
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    wait_for_crate_count(&server, "core_lib", 1);
    wait_for_crate_count(&server, "app", 2);
}

#[test]
fn crate_pulling_in_a_file_that_is_the_same_is_shared() {
    if skip_slow_tests() {
        return;
    }

    let fixture = pulling_in_fixture("pub fn shared() {}", "pub fn shared() {}");
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    wait_for_crate_count(&server, "core_lib", 1);
    wait_for_crate_count(&server, "app", 1);
}

#[test]
fn rename_from_a_worktree_edits_the_worktree_only() {
    if skip_slow_tests() {
        return;
    }

    let server = worktree_server("pub fn run() -> u32 { core_lib::answer() + 1 }");
    let edit = server.send_request::<lsp_types::RenameRequest>(lsp_types::RenameParams {
        text_document_position_params: position(&server, "wt/core_lib/src/lib.rs", 0, 8),
        new_name: "reply".to_owned(),
        work_done_progress_params: Default::default(),
    });
    let edit = edit.to_string();
    assert!(edit.contains("/wt/core_lib/src/lib.rs"), "{edit}");
    assert!(edit.contains("/wt/app/src/lib.rs"), "{edit}");
    assert!(!edit.contains("/base/"), "{edit}");
}

#[test]
fn diagnostics_of_a_shared_crate_reach_the_worktree_client_under_its_path() {
    if skip_slow_tests() {
        return;
    }

    let broken = "pub fn answer() -> u32 { \"no\" }";
    let fixture = CHECKOUT_AND_WORKTREE
        .replace("$APP", "pub fn run() -> u32 { core_lib::answer() }")
        .replace("pub fn answer() -> u32 { 42 }", broken);
    let multi_server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .multi_server();
    let base_client = multi_server.connect(rust_analyzer::ClientId(1));
    base_client.wait_until_workspace_is_loaded();

    let worktree_client = multi_server.connect_with_encoding(rust_analyzer::ClientId(2), None);
    let params: lsp_types::InitializeParams = serde_json::from_value(serde_json::json!({
        "rootUri": worktree_client.doc_id("wt").uri,
        "capabilities": {},
    }))
    .unwrap();
    worktree_client.send_request::<lsp_types::InitializeRequest>(params);
    worktree_client
        .notification::<lsp_types::InitializedNotification>(lsp_types::InitializedParams {});

    let doc_id = worktree_client.doc_id("wt/core_lib/src/lib.rs");
    let text = std::fs::read_to_string(doc_id.uri.to_file_path().unwrap()).unwrap();
    worktree_client.notification::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text,
        },
    });

    let diagnostics = worktree_client.wait_for_diagnostics();
    assert_eq!(diagnostics.uri, doc_id.uri);
    assert_eq!(diagnostics.version, Some(1));

    base_client.shutdown_and_exit();
    worktree_client.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn file_dropped_from_the_worktrees_crate_is_not_answered_from_the_base_checkout() {
    if skip_slow_tests() {
        return;
    }

    // The worktree no longer declares `mod extra`, but still has the unchanged file
    let fixture = CHECKOUT_AND_WORKTREE
        .replace("$APP", "pub fn run() -> u32 { core_lib::answer() }")
        .replacen(
            "//- /base/core_lib/src/lib.rs\npub fn answer() -> u32 { 42 }",
            "//- /base/core_lib/src/lib.rs\npub mod extra;\npub fn answer() -> u32 { 42 }",
            1,
        )
        + "//- /base/core_lib/src/extra.rs\npub fn extra() {}\n\n//- /wt/core_lib/src/extra.rs\npub fn extra() {}\n\n";
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();
    wait_for_crate_count(&server, "core_lib", 2);

    let hover = |path: &str| {
        server
            .send_request::<HoverRequest>(HoverParams {
                text_document_position_params: position(&server, path, 0, 8),
                work_done_progress_params: Default::default(),
            })
            .to_string()
    };
    assert!(hover("base/core_lib/src/extra.rs").contains("core_lib::extra"));
    assert!(!hover("wt/core_lib/src/extra.rs").contains("core_lib::extra"));
}

#[test]
fn diagnostics_of_a_shared_crate_reach_an_open_worktree_document() {
    if skip_slow_tests() {
        return;
    }

    // The client works in a directory that contains both checkouts
    let fixture = CHECKOUT_AND_WORKTREE
        .replace("$APP", "pub fn run() -> u32 { core_lib::answer() }")
        .replace("pub fn answer() -> u32 { 42 }", "pub fn answer() -> u32 { \"no\" }");
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();
    wait_for_crate_count(&server, "core_lib", 1);

    let doc_id = server.doc_id("wt/core_lib/src/lib.rs");
    let text = std::fs::read_to_string(doc_id.uri.to_file_path().unwrap()).unwrap();
    server.notification::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text,
        },
    });

    let diagnostics = server.wait_for_diagnostics();
    assert_eq!(diagnostics.uri, doc_id.uri);
    assert_eq!(diagnostics.version, Some(1));
}

#[test]
fn crate_pulling_in_a_file_that_pulls_in_a_file_that_differs_is_not_shared() {
    if skip_slow_tests() {
        return;
    }

    // `shared.rs` is the same in both, but what it includes is not
    let fixture = pulling_in_fixture("include!(\"nested.rs\");", "include!(\"nested.rs\");")
        + "//- /base/nested.rs\npub fn nested() {}\n\n//- /wt/nested.rs\npub fn nested() -> u8 { 1 }\n\n";
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();

    wait_for_crate_count(&server, "core_lib", 1);
    wait_for_crate_count(&server, "app", 2);
}

#[test]
fn editing_a_pulled_in_file_in_the_worktree_stops_sharing_the_crate() {
    if skip_slow_tests() {
        return;
    }

    let fixture = pulling_in_fixture("pub fn shared() {}", "pub fn shared() {}");
    let server = Project::with_fixture(&fixture)
        .with_config(share_worktrees())
        .root("base")
        .root("wt")
        .server()
        .wait_until_workspace_is_loaded();
    wait_for_crate_count(&server, "app", 1);

    // Only the worktree's copy of the file gets opened and edited
    let doc_id = server.doc_id("wt/shared.rs");
    server.notification::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri,
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn shared() -> u8 { 1 }\n".to_owned(),
        },
    });
    wait_for_crate_count(&server, "app", 2);
}
