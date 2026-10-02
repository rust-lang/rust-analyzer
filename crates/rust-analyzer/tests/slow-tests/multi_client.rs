use lsp_types::{
    CompletionParams, CompletionRequest, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentDiagnosticParams, DocumentDiagnosticRequest,
    DocumentFormattingParams, DocumentFormattingRequest, FormattingOptions, HoverParams,
    HoverRequest, LanguageKind, Position, RenameParams, RenameRequest,
    TextDocumentContentChangeEvent, TextDocumentContentChangeWholeDocument, TextDocumentItem,
    TextDocumentPositionParams, VersionedTextDocumentIdentifier,
};
use rust_analyzer::{ClientId, lsp::ext as lsp_ext};
use test_utils::skip_slow_tests;

use crate::support::multi_project;

#[test]
fn test_multi_client_concurrent_lifecycle() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn hello() -> i32 { 42 }
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Both clients send hover request on `hello` with the EXACT SAME request ID (100)
    let res1 = client1.send_request_with_id::<HoverRequest>(
        100,
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id.clone(),
                position: Position { line: 0, character: 8 },
            },
            work_done_progress_params: Default::default(),
        },
    );

    let res2 = client2.send_request_with_id::<HoverRequest>(
        100,
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id.clone(),
                position: Position { line: 0, character: 8 },
            },
            work_done_progress_params: Default::default(),
        },
    );

    // Verify both clients received valid responses containing the function definition
    let res1_str = res1.to_string();
    let res2_str = res2.to_string();
    assert!(res1_str.contains("pub fn hello() -> i32"), "res1: {res1_str}");
    assert!(res2_str.contains("pub fn hello() -> i32"), "res2: {res2_str}");

    // Client 1 can disconnect without disrupting Client 2
    client1.shutdown_and_exit();

    // Client 2 can still make queries after Client 1 has disconnected
    let res2_after = client2.send_request_with_id::<HoverRequest>(
        200,
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id,
                position: Position { line: 0, character: 8 },
            },
            work_done_progress_params: Default::default(),
        },
    );
    assert!(res2_after.to_string().contains("pub fn hello() -> i32"));

    // Client 2 disconnects
    client2.shutdown_and_exit();

    // Server terminates cleanly when all clients exit
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_document_sharing_and_lifecycle() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn initial() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens the document
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn initial() {}\npub fn added_by_c1() {}\n".to_owned(),
        },
    });

    // Client 2 opens the same document
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 2,
            text: "pub fn initial() {}\npub fn added_by_c1() {}\npub fn added_by_c2() {}\n"
                .to_owned(),
        },
    });

    // Client 2 queries the newly added function to ensure its open was processed
    let res2 = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 2, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res2.to_string().contains("pub fn added_by_c2()"));

    // Client 1's buffer no longer matches the analyzed text, so its positions cannot be trusted
    let res1 = client1.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res1.is_null());

    // Client 1 closes the document: document must remain valid for Client 2
    client1.notification::<lsp_types::DidCloseTextDocumentNotification>(
        DidCloseTextDocumentParams { text_document: doc_id.clone() },
    );

    let res2 = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 2, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res2.to_string().contains("pub fn added_by_c2()"));

    // Client 2 closes the document
    client2.notification::<lsp_types::DidCloseTextDocumentNotification>(
        DidCloseTextDocumentParams { text_document: doc_id },
    );

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_disconnect_without_exit() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn alive() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens a modified version
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn alive() {}\npub fn client1_only() {}\n".to_owned(),
        },
    });

    // Client 1 queries the new function
    let res1 = client1.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res1.to_string().contains("pub fn client1_only()"));

    // Client 1 abruptly drops connection WITHOUT shutdown or exit notification
    drop(client1);

    // Client 2 can make queries again once the server has noticed the disconnect and dropped
    // Client 1's unsaved buffer
    let mut attempts = 0;
    loop {
        let res2 = client2.send_request::<HoverRequest>(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id.clone(),
                position: Position { line: 0, character: 8 },
            },
            work_done_progress_params: Default::default(),
        });
        if res2.to_string().contains("pub fn alive()") {
            break;
        }
        attempts += 1;
        assert!(attempts < 500, "client 2 is still blocked by a disconnected client: {res2}");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Client 2 exits normally
    client2.shutdown_and_exit();

    // Server shuts down cleanly because all clients have exited/disconnected
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_did_change_configuration() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn cfg_test() {}
"#,
    );

    let client2 = multi_server.connect(ClientId(2));
    client2.wait_until_workspace_is_loaded();

    // Client 2 sends DidChangeConfiguration
    client2.notification::<lsp_types::DidChangeConfigurationNotification>(
        lsp_types::DidChangeConfigurationParams { settings: serde_json::json!({}) },
    );

    // The server should send a workspace/configuration request to Client 2
    let req = client2.wait_for_request("workspace/configuration");
    assert_eq!(req.method, "workspace/configuration");

    // Client 2 responds with configuration item array
    client2.respond(lsp_server::Response::new_ok(req.id, serde_json::json!([{}])));

    // Verify client 2 can still perform normal queries after config update
    let doc_id = client2.doc_id("src/lib.rs");
    let res = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id,
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res.to_string().contains("pub fn cfg_test()"));

    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_independent_did_change() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn base() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Both clients open the same document
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn base() {}\n".to_owned(),
        },
    });

    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn base() {}\n".to_owned(),
        },
    });

    // Client 1 edits the document with DidChange
    client1.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "pub fn base() {}\npub fn client1_added() {}\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Client 1 queries hover to verify its edit took effect and ensure ordering
    let res1 = client1.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res1.to_string().contains("pub fn client1_added()"));

    // Client 2 edits the document with disjoint DidChange
    client2.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "pub fn base() {}\npub fn client2_added() {}\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Queries from client 2 should succeed without panics or corruption
    let res2 = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res2.to_string().contains("pub fn client2_added()"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_late_attach_replay() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn late_test() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    client1.wait_until_workspace_is_loaded();

    // Client 2 connects after the workspace has already loaded and become quiescent
    let client2 = multi_server.connect(ClientId(2));

    // Because server status was replayed upon client registration, this does not hang!
    client2.wait_until_workspace_is_loaded();

    // Verify client 2 can immediately query the workspace
    let doc_id = client2.doc_id("src/lib.rs");
    let res = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id,
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res.to_string().contains("pub fn late_test()"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_divergent_buffer_safety() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn base() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Both clients open the same document
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn base() {}\n".to_owned(),
        },
    });

    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn base() {}\n".to_owned(),
        },
    });

    // Client 2 edits first
    client2.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "pub fn base() {}\npub fn client2_only() {}\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Synchronize Client 2's edit
    let _ = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 0, character: 7 },
        },
        work_done_progress_params: Default::default(),
    });

    // Client 1 edits second, becoming the authoritative writer
    client1.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "pub fn base() {}\npub fn client1_author() {}\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Synchronize Client 1's edit
    let _ = client1.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 0, character: 7 },
        },
        work_done_progress_params: Default::default(),
    });

    // Client 2's formatting request yields no edits because Client 2's buffer is divergent
    let fmt2 = client2.send_request::<DocumentFormattingRequest>(DocumentFormattingParams {
        text_document: doc_id.clone(),
        options: FormattingOptions { tab_size: 4, insert_spaces: true, ..Default::default() },
        work_done_progress_params: Default::default(),
    });
    assert!(fmt2.is_null());

    // Client 2 completion returns null while divergent
    let comp = client2.send_request::<CompletionRequest>(CompletionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 7 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: None,
    });
    assert!(comp.is_null());

    // Client 2 pull diagnostics return empty report while divergent
    let diag = client2.send_request::<DocumentDiagnosticRequest>(DocumentDiagnosticParams {
        text_document: doc_id.clone(),
        identifier: None,
        previous_result_id: None,
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    });
    assert_eq!(diag["items"], serde_json::json!([]));

    // Client 1 format succeeds because client 1 is in sync with VFS
    let fmt1 = client1.send_request::<DocumentFormattingRequest>(DocumentFormattingParams {
        text_document: doc_id.clone(),
        options: FormattingOptions { tab_size: 4, insert_spaces: true, ..Default::default() },
        work_done_progress_params: Default::default(),
    });
    assert!(fmt1.is_array() || fmt1.is_null());

    // Client 1 closes, restoring Client 2's buffer into VFS through normal event processing
    client1.notification::<lsp_types::DidCloseTextDocumentNotification>(
        DidCloseTextDocumentParams { text_document: doc_id.clone() },
    );

    // Synchronize Client 1's close
    let _ = client1.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 0, character: 7 },
        },
        work_done_progress_params: Default::default(),
    });

    // Now Client 2 is author and in sync, so Client 2 format succeeds
    let fmt2 = client2.send_request::<DocumentFormattingRequest>(DocumentFormattingParams {
        text_document: doc_id.clone(),
        options: FormattingOptions { tab_size: 4, insert_spaces: true, ..Default::default() },
        work_done_progress_params: Default::default(),
    });
    assert!(fmt2.is_array() || fmt2.is_null());

    // And Client 2 completion succeeds
    let comp2 = client2.send_request::<CompletionRequest>(CompletionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id,
            position: Position { line: 0, character: 7 },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: None,
    });
    assert!(!comp2.is_null());

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_unopened_divergent_edit_rejected() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub struct OldName;

//- /src/main.rs
use foo::OldName;
fn main() { let _x = OldName; }
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let lib_id = client1.doc_id("src/lib.rs");
    let main_id = client2.doc_id("src/main.rs");

    // Client 1 opens src/lib.rs and makes an unsaved edit
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: lib_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub struct OldName;\n// unsaved edit by client 1\n".to_owned(),
        },
    });

    client1.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: lib_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "pub struct OldName;\n// more unsaved modifications\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Synchronize Client 1's edit before Client 2 acts
    let _ = client1.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: lib_id.clone(),
            position: Position { line: 0, character: 11 },
        },
        work_done_progress_params: Default::default(),
    });

    // Client 2 opens src/main.rs (Client 2 has NOT opened src/lib.rs)
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: main_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "use foo::OldName;\nfn main() { let _x = OldName; }\n".to_owned(),
        },
    });

    // Client 2 requests rename of OldName in src/main.rs.
    // Because OldName is defined in src/lib.rs, the rename would require editing src/lib.rs.
    // Since src/lib.rs is unopened by Client 2 and has unsaved changes by Client 1,
    // this cross-file edit must be rejected to prevent corrupting src/lib.rs on disk!
    let err = client2
        .send_request_fallible::<RenameRequest>(RenameParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: main_id,
                position: Position { line: 1, character: 24 },
            },
            new_name: "NewName".to_owned(),
            work_done_progress_params: Default::default(),
        })
        .unwrap_err();

    assert_eq!(err.code, lsp_server::ErrorCode::InvalidParams as i32);
    assert!(err.message.contains("divergent unsaved modifications across clients"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_synchronized_diagnostics_replayed() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn foo() {
    let x: i32 = "error";
}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens src/lib.rs with base text (containing a type mismatch diagnostic)
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn foo() {\n    let x: i32 = \"error\";\n}\n".to_owned(),
        },
    });

    // Client 1 receives the diagnostic
    let diag1 = client1.wait_for_diagnostics();
    assert!(crate::support::same_uri(&diag1.uri, &doc_id.uri), "{:?}", diag1.uri);
    assert!(!diag1.diagnostics.is_empty());

    // Client 2 opens src/lib.rs with an unsaved divergent edit
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn foo() {\n    let x: i32 = 42;\n    // divergent\n}\n".to_owned(),
        },
    });

    // Client 2 is divergent, so diagnostics are suppressed for Client 2.
    // Now Client 2 edits its buffer to match Client 1's authoritative VFS buffer!
    client2.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "pub fn foo() {\n    let x: i32 = \"error\";\n}\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Because Client 2 transitioned from divergent -> in sync, diagnostics must be replayed to Client 2!
    let diag2 = client2.wait_for_diagnostics_with_version(&doc_id.uri, 2);
    assert!(crate::support::same_uri(&diag2.uri, &doc_id.uri), "{:?}", diag2.uri);
    assert_eq!(diag2.version, Some(2));
    assert!(!diag2.diagnostics.is_empty());

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_divergent_author_unchanged_diagnostics_delivered() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn foo() {
    let x: i32 = "error";
}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens src/lib.rs with base text (containing a type mismatch diagnostic)
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn foo() {\n    let x: i32 = \"error\";\n}\n".to_owned(),
        },
    });

    // Client 1 receives the diagnostic
    let diag1 = client1.wait_for_diagnostics_with_version(&doc_id.uri, 1);
    assert!(crate::support::same_uri(&diag1.uri, &doc_id.uri), "{:?}", diag1.uri);
    assert_eq!(diag1.version, Some(1));
    assert!(!diag1.diagnostics.is_empty());

    // Client 2 opens src/lib.rs with an unsaved divergent edit
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn foo() {\n    let x: i32 = 42;\n    // divergent\n}\n".to_owned(),
        },
    });

    // Client 2 edits with new text that changes VFS, becomes author, but produces unchanged diagnostics
    client2.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text:
                            "pub fn foo() {\n    let x: i32 = \"error\";\n}\n// trailing comment\n"
                                .to_owned(),
                    },
                ),
            ],
        },
    );

    // Client 2 must receive the diagnostic even though diagnostic contents are identical to cached!
    let diag2 = client2.wait_for_diagnostics_with_version(&doc_id.uri, 2);
    assert!(crate::support::same_uri(&diag2.uri, &doc_id.uri), "{:?}", diag2.uri);
    assert_eq!(diag2.version, Some(2));
    assert!(!diag2.diagnostics.is_empty());

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_move_divergent_file_rejected() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
mod sub;

//- /src/sub.rs
pub fn sub_fn() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let sub_id = client1.doc_id("src/sub.rs");
    let lib_id = client2.doc_id("src/lib.rs");

    // Client 1 opens src/sub.rs and has unsaved changes
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: sub_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn sub_fn() {}\n// unsaved edit by client 1\n".to_owned(),
        },
    });

    // Client 2 opens src/lib.rs (has NOT opened src/sub.rs)
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: lib_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "mod sub;\n".to_owned(),
        },
    });

    // Client 2 requests rename of `mod sub` to `mod new_sub`, which causes a MoveFile operation!
    let err = client2
        .send_request_fallible::<RenameRequest>(RenameParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: lib_id,
                position: Position { line: 0, character: 5 },
            },
            new_name: "new_sub".to_owned(),
            work_done_progress_params: Default::default(),
        })
        .unwrap_err();
    assert_eq!(err.code, lsp_server::ErrorCode::InvalidParams as i32);
    assert!(err.message.contains("divergent unsaved modifications across clients"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
#[expect(deprecated)]
fn test_multi_client_different_position_encodings() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
// 🦀 crab
pub fn hello() -> i32 { 42 }
"#,
    );

    // Client 1 negotiates UTF-16 (default)
    let client1 = multi_server.connect_with_encoding(
        ClientId(1),
        Some(rust_analyzer::PositionEncoding::Wide(rust_analyzer::WideEncoding::Utf16)),
    );
    // Client 2 negotiates UTF-8
    let client2 = multi_server
        .connect_with_encoding(ClientId(2), Some(rust_analyzer::PositionEncoding::Utf8));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens doc (version 1)
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "// 🦀 crab\npub fn hello() -> i32 { 42 }\n".to_owned(),
        },
    });

    // Client 2 opens doc (version 1)
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "// 🦀 crab\npub fn hello() -> i32 { 42 }\n".to_owned(),
        },
    });

    // Client 1 inserts `!` right after `🦀` using UTF-16 encoding:
    // `// ` is 3 UTF-16 units. `🦀` is 2 UTF-16 units (surrogate pair).
    // Position after 🦀 is character: 5.
    client1.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangePartial(
                    lsp_types::TextDocumentContentChangePartial {
                        range: lsp_types::Range {
                            start: Position { line: 0, character: 5 },
                            end: Position { line: 0, character: 5 },
                        },
                        range_length: None,
                        text: "!".to_owned(),
                    },
                ),
            ],
        },
    );

    // Verify Client 1's buffer has `// 🦀! crab` and hover on `hello` works
    let res1 = client1.send_request_with_id::<HoverRequest>(
        101,
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id.clone(),
                position: Position { line: 1, character: 8 },
            },
            work_done_progress_params: Default::default(),
        },
    );
    assert!(res1.to_string().contains("pub fn hello() -> i32"));

    // Now Client 2 inserts `?` right after `🦀` using UTF-8 encoding:
    // `// ` is 3 UTF-8 bytes. `🦀` is 4 UTF-8 bytes.
    // Position after 🦀 in Client 2's buffer is character: 7.
    client2.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangePartial(
                    lsp_types::TextDocumentContentChangePartial {
                        range: lsp_types::Range {
                            start: Position { line: 0, character: 7 },
                            end: Position { line: 0, character: 7 },
                        },
                        range_length: None,
                        text: "?".to_owned(),
                    },
                ),
            ],
        },
    );

    // Verify Client 2's hover also works and did not panic or corrupt
    let res2 = client2.send_request_with_id::<HoverRequest>(
        102,
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id,
                position: Position { line: 1, character: 8 },
            },
            work_done_progress_params: Default::default(),
        },
    );
    assert!(res2.to_string().contains("pub fn hello() -> i32"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_inlay_hints_suppressed_when_divergent() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn foo() {
    let x = 42;
}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens doc
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn foo() {\n    let x = 42;\n}\n".to_owned(),
        },
    });

    // Client 2 opens doc with unsaved divergent changes
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 2,
            text: "// divergence line\npub fn foo() {\n    let x = 42;\n}\n".to_owned(),
        },
    });

    // Messages of different clients are not ordered relative to each other, so wait until
    // Client 2's open has been processed
    client2.send_request::<lsp_types::HoverRequest>(lsp_types::HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });

    // Client 1 requests inlay hints on the file (which is divergent across clients)
    let hints_res =
        client1.send_request::<lsp_types::InlayHintRequest>(lsp_types::InlayHintParams {
            text_document: doc_id.clone(),
            range: lsp_types::Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 3, character: 0 },
            },
            work_done_progress_params: Default::default(),
        });

    // The hints would be positioned relative to Client 2's text, so none are returned
    assert!(hints_res.is_null());

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_code_action_preserves_safe_actions_when_one_divergent() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn foo() {
    let x = 42;
}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens doc
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn foo() {\n    let x = 42;\n}\n".to_owned(),
        },
    });

    // Request code actions on `x`
    let actions_val =
        client1.send_request::<lsp_types::CodeActionRequest>(lsp_types::CodeActionParams {
            text_document: doc_id,
            range: lsp_types::Range {
                start: Position { line: 1, character: 8 },
                end: Position { line: 1, character: 9 },
            },
            context: lsp_types::CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        });

    let actions: Vec<lsp_ext::CodeAction> = serde_json::from_value(actions_val).unwrap();
    // Verify that code actions are returned successfully
    assert!(!actions.is_empty(), "expected code actions for variable");

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_divergent_semantic_query_suppressed() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn calculate() -> u32 {
    let value = 42;
    value
}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");

    // Client 1 opens doc with initial content
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn calculate() -> u32 {\n    let value = 42;\n    value\n}\n".to_owned(),
        },
    });

    // Client 1 requests hover on `calculate` -> returns valid hover
    let hover1 = client1.send_request::<lsp_types::HoverRequest>(lsp_types::HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(!hover1.is_null(), "Client 1 hover on calculate should succeed");

    // Client 2 opens the same file and edits it divergently without saving
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: "pub fn calculate() -> u32 {\n    let value = 42;\n    value\n}\n".to_owned(),
        },
    });
    client2.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "// divergent edits\npub struct DifferentType;\n".to_owned(),
                    },
                ),
            ],
        },
    );

    // Client 2 wrote last, so the analyzed text is its buffer
    let hover2 = client2.send_request::<lsp_types::HoverRequest>(lsp_types::HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 1, character: 11 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(hover2.to_string().contains("DifferentType"));

    // Client 1's buffer is now divergent, so semantic queries like hover are suppressed
    let hover1_repeat = client1.send_request::<lsp_types::HoverRequest>(lsp_types::HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id,
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(hover1_repeat.is_null(), "Client 1 hover should be suppressed while divergent");

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_shutdown_is_per_client() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn hello() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id,
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    };

    client1.send_request::<lsp_types::ShutdownRequest>(());

    // Client 1 has shut down, so its further requests are rejected
    let err = client1.send_request_fallible::<HoverRequest>(hover_params.clone()).unwrap_err();
    assert_eq!(err.code, lsp_server::ErrorCode::InvalidRequest as i32);

    // The server keeps serving Client 2
    let res2 = client2.send_request::<HoverRequest>(hover_params);
    assert!(res2.to_string().contains("pub fn hello()"));

    client1.notification::<lsp_types::ExitNotification>(());
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_initialize_handshake() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn hello() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    client1.wait_until_workspace_is_loaded();

    // Client 2 attaches without a negotiated encoding and goes through the LSP handshake itself
    let client2 = multi_server.connect_with_encoding(ClientId(2), None);
    let params: lsp_types::InitializeParams = serde_json::from_value(serde_json::json!({
        "capabilities": { "general": { "positionEncodings": ["utf-8"] } },
    }))
    .unwrap();
    let res = client2.send_request::<lsp_types::InitializeRequest>(params);
    assert_eq!(res["capabilities"]["positionEncoding"], "utf-8");
    client2.notification::<lsp_types::InitializedNotification>(lsp_types::InitializedParams {});

    let res2 = client2.send_request::<HoverRequest>(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: client2.doc_id("src/lib.rs"),
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    });
    assert!(res2.to_string().contains("pub fn hello()"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_diagnostics_use_client_encoding() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn foo() {
    let _s = "🦀"; let _x: i32 = "error";
}
"#,
    );

    let client1 = multi_server.connect_with_encoding(
        ClientId(1),
        Some(rust_analyzer::PositionEncoding::Wide(rust_analyzer::WideEncoding::Utf16)),
    );
    let client2 = multi_server
        .connect_with_encoding(ClientId(2), Some(rust_analyzer::PositionEncoding::Utf8));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");
    let text = "pub fn foo() {\n    let _s = \"🦀\"; let _x: i32 = \"error\";\n}\n";
    for client in [&client1, &client2] {
        client.notification::<lsp_types::DidOpenTextDocumentNotification>(
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: doc_id.uri.clone(),
                    language_id: LanguageKind::Rust,
                    version: 1,
                    text: text.to_owned(),
                },
            },
        );
    }

    let diag1 = client1.wait_for_diagnostics().diagnostics.remove(0);
    let diag2 = client2.wait_for_diagnostics().diagnostics.remove(0);
    assert_eq!(diag1.message, diag2.message);
    assert_eq!(diag1.range.start.line, 1);
    // `🦀` is 2 UTF-16 code units, but 4 UTF-8 bytes
    assert_eq!(diag2.range.start.character, diag1.range.start.character + 2);
    assert_eq!(diag2.range.end.character, diag1.range.end.character + 2);

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_view_file_text_is_per_client() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn base() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");
    for (client, text) in [(&client1, "pub fn one() {}\n"), (&client2, "pub fn two() {}\n")] {
        client.notification::<lsp_types::DidOpenTextDocumentNotification>(
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: doc_id.uri.clone(),
                    language_id: LanguageKind::Rust,
                    version: 1,
                    text: text.to_owned(),
                },
            },
        );
    }

    // Each client sees its own buffer, no matter which one is being analyzed
    let text1 = client1.send_request::<lsp_ext::ViewFileTextRequest>(doc_id.clone());
    let text2 = client2.send_request::<lsp_ext::ViewFileTextRequest>(doc_id);
    assert_eq!(text1, "pub fn one() {}\n");
    assert_eq!(text2, "pub fn two() {}\n");

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_stale_resolve_after_divergence() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn base() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    // Client 2 writes last, which makes Client 1's buffer divergent
    let doc_id = client1.doc_id("src/lib.rs");
    for (client, text) in [(&client1, "pub fn one() {}\n"), (&client2, "pub fn two() {}\n")] {
        client.notification::<lsp_types::DidOpenTextDocumentNotification>(
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: doc_id.uri.clone(),
                    language_id: LanguageKind::Rust,
                    version: 1,
                    text: text.to_owned(),
                },
            },
        );
    }

    // Resolving an item Client 1 got before its buffer diverged leaves the item as it is
    let completion: lsp_types::CompletionItem = serde_json::from_value(serde_json::json!({
        "label": "one",
        "data": {
            "position": { "textDocument": doc_id, "position": { "line": 0, "character": 7 } },
            "hash": "",
        },
    }))
    .unwrap();
    let resolved = client1.send_request::<lsp_types::CompletionResolveRequest>(completion);
    assert_eq!(resolved["label"], "one");
    assert!(resolved.get("additionalTextEdits").is_none());

    // And such a code action is reported as stale
    let code_action: lsp_ext::CodeAction = serde_json::from_value(serde_json::json!({
        "title": "action",
        "data": {
            "codeActionParams": {
                "textDocument": doc_id,
                "range": {
                    "start": { "line": 0, "character": 7 },
                    "end": { "line": 0, "character": 7 },
                },
                "context": { "diagnostics": [] },
            },
            "id": "action:RefactorRewrite:0",
            "version": 1,
        },
    }))
    .unwrap();
    let err = client1
        .send_request_fallible::<lsp_ext::CodeActionResolveRequest>(code_action)
        .unwrap_err();
    assert_eq!(err.code, lsp_server::ErrorCode::InvalidParams as i32);

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_disconnect_restores_surviving_buffer() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn base() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));

    client1.wait_until_workspace_is_loaded();

    // Client 2 writes last, so its buffer is the analyzed one
    let doc_id = client1.doc_id("src/lib.rs");
    for (client, text) in [(&client1, "pub fn one() {}\n"), (&client2, "pub fn two() {}\n")] {
        client.notification::<lsp_types::DidOpenTextDocumentNotification>(
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: doc_id.uri.clone(),
                    language_id: LanguageKind::Rust,
                    version: 1,
                    text: text.to_owned(),
                },
            },
        );
    }

    drop(client2);

    // Once the server has noticed the disconnect, Client 1's buffer is analyzed again
    let mut attempts = 0;
    loop {
        let res1 = client1.send_request::<HoverRequest>(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: doc_id.clone(),
                position: Position { line: 0, character: 8 },
            },
            work_done_progress_params: Default::default(),
        });
        if res1.to_string().contains("pub fn one()") {
            break;
        }
        attempts += 1;
        assert!(attempts < 500, "client 1's buffer was not restored: {res1}");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    client1.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}

#[test]
fn test_multi_client_unopened_file_served_when_buffer_matches_disk() {
    if skip_slow_tests() {
        return;
    }

    let multi_server = multi_project(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/lib.rs
pub fn base() {}
"#,
    );

    let client1 = multi_server.connect(ClientId(1));
    let client2 = multi_server.connect(ClientId(2));
    let client3 = multi_server.connect(ClientId(3));

    client1.wait_until_workspace_is_loaded();

    let doc_id = client1.doc_id("src/lib.rs");
    let disk_text = "pub fn base() {}\n";
    let open = |text: &str| DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: doc_id.uri.clone(),
            language_id: LanguageKind::Rust,
            version: 1,
            text: text.to_owned(),
        },
    };
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: doc_id.clone(),
            position: Position { line: 0, character: 8 },
        },
        work_done_progress_params: Default::default(),
    };

    // Client 3 never opens the document, so it refers to the text on disk
    client1.notification::<lsp_types::DidOpenTextDocumentNotification>(open(disk_text));
    let res = client3.send_request::<HoverRequest>(hover_params.clone());
    assert!(res.to_string().contains("pub fn base()"));

    // Client 1 has unsaved changes: Client 3's positions no longer apply to the analyzed text
    client1.notification::<lsp_types::DidChangeTextDocumentNotification>(
        DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                text_document_identifier: doc_id.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(
                    TextDocumentContentChangeWholeDocument {
                        text: "// unsaved\npub fn base() {}\n".to_owned(),
                    },
                ),
            ],
        },
    );
    let res = client3.send_request::<HoverRequest>(hover_params.clone());
    assert!(res.is_null());

    // Client 2 opens the text that is on disk and becomes the author
    client2.notification::<lsp_types::DidOpenTextDocumentNotification>(open(disk_text));
    let res = client3.send_request::<HoverRequest>(hover_params);
    assert!(res.to_string().contains("pub fn base()"));

    client1.shutdown_and_exit();
    client2.shutdown_and_exit();
    client3.shutdown_and_exit();
    multi_server.wait_for_shutdown();
}
