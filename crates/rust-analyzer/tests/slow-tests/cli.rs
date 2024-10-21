use std::process::Command;

use test_utils::skip_slow_tests;

const RA_BIN: &str = "../../target/debug/rust-analyzer";

// If you choose to change the behavior of the lsif command and therefore
// modify or remove this test, please inform the ferrocene/needy maintainers by
// opening an issue at https://github.com/ferrocene/needy.
#[test]
fn lsif_contains_generated_constant() {
    if skip_slow_tests() {
        return;
    }

    // Arrange
    let mut cmd = Command::new(RA_BIN);
    cmd.args(["lsif", "tests/lsif-test-crate"]);

    // Act
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Assert
    assert!(stdout.contains("REQ_001"));
    assert!(stdout.contains("lsif_test_crate"));

    assert!(stdout.contains("REQ_002"));
    assert!(stdout.contains("lsif_test_crate::tests"));

    assert!(stdout.contains("encoded_data"));
}
