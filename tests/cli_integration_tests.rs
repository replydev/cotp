#[cfg(not(target_os = "windows"))] // TODO, Integration tests currently does not work on Windows
mod cli_integration_test {

    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::str::{is_empty, is_match, starts_with};

    #[test]
    fn test_version_subcommand() {
        // Arrange / Act
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command.arg("--version").assert();

        // Assert
        assertion
            .success()
            .stdout(is_match("^cotp \\d+\\.\\d+\\.\\d+(?:-DEBUG-.+)?").unwrap())
            .stderr(is_empty());
    }

    #[test]
    fn test_delete_without_selector_is_rejected() {
        // Arrange / Act: no --index, --issuer or --label is provided, so clap
        // must reject the invocation instead of letting the matcher fall
        // through to an empty-string match deleting the first element.
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command.arg("delete").assert();

        // Assert
        assertion
            .failure()
            .stderr(is_match("required arguments were not provided").unwrap());
    }

    #[test]
    fn test_help_subcommand() {
        // Arrange / Act
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command.arg("--help").assert();

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
