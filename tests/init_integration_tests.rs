#[cfg(not(target_os = "windows"))] // TODO, Integration tests currently does not work on Windows
mod init_integration_tests {
    use assert_cmd::cargo::cargo_bin_cmd;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use predicates::str::contains;

    const FIXTURE_DIR: &str = "test_samples/cli_integration_test";
    const FIXTURE_NAME: &str = "empty_database";
    const FIXTURE_PASSWORD: &str = "12345678";

    /// Regression test: when `--database-path` is a bare relative filename, an
    /// existing database must be loaded instead of being treated as a first run
    /// (which used to prompt for a new password and overwrite it with an empty
    /// database).
    #[test]
    fn existing_database_with_bare_relative_path_survives() {
        // Arrange: put an existing populated-able database in a temp working dir
        let temp = TempDir::new().unwrap();
        temp.copy_from(FIXTURE_DIR, &[FIXTURE_NAME]).unwrap();

        // Act 1: first invocation with a bare relative -d filename adds an element
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command
            .current_dir(temp.path())
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(FIXTURE_NAME)
            .arg("add")
            .arg("--label")
            .arg("relative-path-test")
            .arg("--secret-stdin")
            .write_stdin(format!("{FIXTURE_PASSWORD}\nAA\n"))
            .assert();
        assertion
            .success()
            .stdout(contains("Modifications have been persisted"));

        // Act 2: second invocation with the same bare relative -d filename must
        // load the existing database (not re-initialize it) and still contain
        // the element added by the first run
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command
            .current_dir(temp.path())
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(FIXTURE_NAME)
            .arg("list")
            .write_stdin(format!("{FIXTURE_PASSWORD}\n"))
            .assert();

        // Assert
        assertion.success().stdout(contains("relative-path-test"));
    }
}
