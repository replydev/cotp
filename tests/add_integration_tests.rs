#[cfg(not(target_os = "windows"))] // TODO, Integration tests currently does not work on Windows
mod add_integration_tests {
    use assert_cmd::Command;
    use predicates::{ord::eq, str::is_empty};
    use test_case::test_case;

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

    #[test_case("-l" ; "Short subcommand")]
    #[test_case("--label" ; "Long subcommand")]
    fn add_with_label_should_work(label_arg: &str) {
        // Arrange / Act
        let assertion = Command::cargo_bin("cotp")
            .unwrap()
            .arg("--password-stdin")
            .arg("--database-path")
            .arg("test_samples/cli_integration_test/empty_database")
            .arg("add")
            .arg(label_arg)
            .arg("test")
            .arg("--secret-stdin")
            .write_stdin(
                "12345678
AA
        ",
            )
            .assert();

        // Assert
        assertion
            .success()
            .stderr(is_empty())
            .stdout(eq("Modifications have been persisted\n"));
    }
}
