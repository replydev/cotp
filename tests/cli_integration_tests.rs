use assert_cmd::Command;
use predicates::str::is_match;

#[test]
fn test_version_subcommand() {
    // Arrange / Act
    let assertion = Command::cargo_bin("cotp")
        .unwrap()
        .arg("--version")
        .assert();

    // Assert
    assertion
        .success()
        .stdout(is_match("^cotp \\d+\\.\\d+\\.\\d+-DEBUG-.+").unwrap());
}
