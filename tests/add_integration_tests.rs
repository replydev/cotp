#[cfg(not(target_os = "windows"))] // TODO, Integration tests currently does not work on Windows
mod add_integration_tests {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::{ord::eq, str::is_empty};
    use test_case::test_case;

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
        // Arrange / Act
        let dir = assert_fs::TempDir::new().unwrap();
        let database = dir.path().join("db.cotp");
        std::fs::copy(
            "test_samples/cli_integration_test/empty_database",
            &database,
        )
        .unwrap();
        let mut command = cargo_bin_cmd!("cotp");
        let assertion = command
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(&database)
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

    #[cfg(unix)]
    #[test]
    fn failed_database_write_preserves_existing_otps() {
        let dir = assert_fs::TempDir::new().unwrap();
        let database = dir.path().join("db.cotp");
        std::fs::copy(
            "test_samples/cli_integration_test/empty_database",
            &database,
        )
        .unwrap();

        // Store an OTP first so that the regression checks preservation of real data.
        cargo_bin_cmd!("cotp")
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(&database)
            .args(["add", "--label", "existing", "--secret-stdin"])
            .write_stdin("12345678\nAA\n")
            .assert()
            .success();
        let original = std::fs::read(&database).unwrap();

        // A file-size limit forces a real partial write failure without filling the disk.
        // Ignoring SIGXFSZ makes the write return an error instead of killing cotp.
        let mut limited = assert_cmd::Command::new("/bin/sh");
        limited
            .args(["-c", "trap '' XFSZ; ulimit -f 1; exec \"$@\"", "sh"])
            .arg(assert_cmd::cargo::cargo_bin!("cotp"))
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(&database)
            .args(["add", "--label", &"x".repeat(8192), "--secret-stdin"])
            .write_stdin("12345678\nAA\n")
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                "An error occurred during database overwriting:",
            ));

        assert_eq!(std::fs::read(&database).unwrap(), original);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        cargo_bin_cmd!("cotp")
            .arg("--password-stdin")
            .arg("--database-path")
            .arg(&database)
            .arg("list")
            .write_stdin("12345678\n")
            .assert()
            .success()
            .stdout(predicates::str::contains("existing"));
    }
}
