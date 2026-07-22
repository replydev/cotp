#[cfg(not(target_os = "windows"))] // TODO, Integration tests currently does not work on Windows
mod add_integration_tests {
    use assert_cmd::cargo::cargo_bin_cmd;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use predicates::{ord::eq, str::is_empty};
    use test_case::test_case;

    const FIXTURE_DIR: &str = "test_samples/cli_integration_test";
    const FIXTURE_NAME: &str = "empty_database";

    /// Copies the committed database fixture into a temporary directory so
    /// tests never mutate files tracked by git
    fn temp_database() -> (TempDir, std::path::PathBuf) {
        let temp = TempDir::new().unwrap();
        temp.copy_from(FIXTURE_DIR, &[FIXTURE_NAME]).unwrap();
        let database_path = temp.child(FIXTURE_NAME).path().to_path_buf();
        (temp, database_path)
    }

    #[test]
    fn add_without_label_should_fail() {
        // Arrange / Act
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command
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
        // Arrange
        let (_temp, database_path) = temp_database();

        // Act
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(database_path)
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
