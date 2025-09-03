#[cfg(not(target_os = "windows"))] // TODO, Integration tests currently does not work on Windows
mod cli_integration_test {

    use assert_cmd::Command;
    use predicates::str::{is_empty, is_match, starts_with};

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
            .stdout(is_match("^cotp \\d+\\.\\d+\\.\\d+-DEBUG-.+").unwrap())
            .stderr(is_empty());
    }

    #[test]
    fn test_help_subcommand() {
        // Arrange / Act
        let assertion = Command::cargo_bin("cotp").unwrap().arg("--help").assert();

        // Assert
        assertion
        .success()
        .stdout(starts_with(
            "Trustworthy, encrypted, command-line TOTP/HOTP authenticator app with import functionality.

Usage: cotp [OPTIONS] [COMMAND]",
        ))
        .stderr(is_empty());
    }
}
