use assert_cmd::Command;
use predicates::{
    ord::eq,
    str::{is_empty, is_match, starts_with},
};

#[test]
fn add_without_label_should_fail() {
    // Arrange / Act
    let assertion = Command::cargo_bin("cotp")
        .unwrap()
        .arg("--database-path")
        .arg("test_samples/cli_integration_test/empty_database")
        .arg("add")
        .assert();

    // Assert
    assertion.failure().code(2).stdout(is_empty()).stderr(eq(
        "error: the following required arguments were not provided:
  --otpuri
  --label <LABEL>

Usage: cotp add --otpuri --label <LABEL>

For more information, try '--help'.
",
    ));
}
