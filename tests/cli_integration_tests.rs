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
    fn test_help_subcommand() {
        // Arrange / Act
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command.arg("--help").assert();

        // Assert
        assertion
            .success()
            .stdout(starts_with(
                "Manage one-time passwords in an encrypted vault

Usage: cotp [OPTIONS] [COMMAND]",
            ))
            .stderr(is_empty());
    }

    #[test]
    fn dashboard_without_terminal_reports_error_once() {
        let dir = assert_fs::TempDir::new().unwrap();
        let database = dir.path().join("db.cotp");
        std::fs::copy(
            "test_samples/cli_integration_test/empty_database",
            &database,
        )
        .unwrap();
        cargo_bin_cmd!("cotp")
            .arg("--database-path")
            .arg(&database)
            .arg("--password-stdin")
            .args(["add", "--label", "existing", "--secret-stdin"])
            .write_stdin("12345678\nAA\n")
            .assert()
            .success();

        cargo_bin_cmd!("cotp")
            .arg("--database-path")
            .arg(&database)
            .arg("--password-stdin")
            .write_stdin("12345678\n")
            .assert()
            .failure()
            .stdout(is_empty())
            .stderr("An error occurred: The dashboard requires terminal input and stderr connected to a terminal. Use cotp list or cotp list --json to show codes without the dashboard.\n");
    }
}
