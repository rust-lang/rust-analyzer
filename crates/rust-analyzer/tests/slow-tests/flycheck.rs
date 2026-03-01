use test_utils::skip_slow_tests;

use crate::support::Project;

#[test]
fn test_flycheck_diagnostics_for_unused_variable() {
    if skip_slow_tests() {
        return;
    }

    let server = Project::with_fixture(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/main.rs
fn main() {
    let x = 1;
}
"#,
    )
    .with_config(serde_json::json!({
        "checkOnSave": true,
    }))
    .server()
    .wait_until_workspace_is_loaded();

    let diagnostics = server.wait_for_diagnostics();
    assert!(
        diagnostics.diagnostics.iter().any(|d| d.message.contains("unused variable")),
        "expected unused variable diagnostic, got: {:?}",
        diagnostics.diagnostics,
    );
}

#[test]
fn test_flycheck_diagnostic_cleared_after_fix() {
    if skip_slow_tests() {
        return;
    }

    let server = Project::with_fixture(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/main.rs
fn main() {
    let x = 1;
}
"#,
    )
    .with_config(serde_json::json!({
        "checkOnSave": true,
    }))
    .server()
    .wait_until_workspace_is_loaded();

    // Wait for the unused variable diagnostic to appear.
    let diagnostics = server.wait_for_diagnostics();
    assert!(
        diagnostics.diagnostics.iter().any(|d| d.message.contains("unused variable")),
        "expected unused variable diagnostic, got: {:?}",
        diagnostics.diagnostics,
    );

    // Fix the code by removing the unused variable.
    server.write_file_and_save("src/main.rs", "fn main() {}\n".to_owned());

    // Wait for diagnostics to be cleared.
    server.wait_for_diagnostics_cleared();
}

#[test]
fn test_flycheck_diagnostic_with_override_command() {
    if skip_slow_tests() {
        return;
    }

    let server = Project::with_fixture(
        r#"
//- /Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /src/main.rs
fn main() {}
"#,
    )
    .with_config(serde_json::json!({
        "checkOnSave": true,
        "check": {
            "overrideCommand": ["rustc", "--error-format=json", "$saved_file"]
        }
    }))
    .server()
    .wait_until_workspace_is_loaded();

    server.write_file_and_save("src/main.rs", "fn main() {\n    let x = 1;\n}\n".to_owned());

    let diagnostics = server.wait_for_diagnostics();
    assert!(
        diagnostics.diagnostics.iter().any(|d| d.message.contains("unused variable")),
        "expected unused variable diagnostic, got: {:?}",
        diagnostics.diagnostics,
    );
}

#[cfg(unix)] // We're using sh -c in this test, so skip windows.
#[test]
fn test_flycheck_multiple_workspaces() {
    if skip_slow_tests() {
        return;
    }

    let server = Project::with_fixture(
        r#"
//- /ws1/Cargo.toml
[package]
name = "foo"
version = "0.0.0"

//- /ws1/src/main.rs
fn main() {}

//- /ws2/Cargo.toml
[package]
name = "bar"
version = "0.0.0"

//- /ws2/src/main.rs
fn main() {}
"#,
    )
    .root("ws1")
    .root("ws2")
    .with_config(serde_json::json!({
        "checkOnSave": true,
        "check": {
            "overrideCommand": ["sh", "-c", "sleep 1 && rustc --error-format=json {saved_file}"],
        }
    }))
    .server()
    .wait_until_workspace_is_loaded();

    server.write_file_and_save("ws1/src/main.rs", "fn main() {\n    let x = 1;\n}\n".to_owned());

    let diag1 = server.wait_for_diagnostics();
    assert!(
        diag1.diagnostics.iter().any(|d| d.message.contains("unused variable")),
        "expected unused variable diagnostic from ws1, got: {:?}",
        diag1.diagnostics,
    );
}
