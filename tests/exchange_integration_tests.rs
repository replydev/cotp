use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn cmd(vault: &Path) -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cotp"));
    cmd.arg("--database-path")
        .arg(vault)
        .args(["--no-input", "--password-stdin"]);
    cmd
}

#[test]
fn encrypted_merge_authenticates_and_dry_run_preserves_destination() {
    let dir = assert_fs::TempDir::new().unwrap();
    let source = dir.path().join("source.cotp");
    let destination = dir.path().join("destination.cotp");
    for path in [&source, &destination] {
        cmd(path)
            .arg("init")
            .write_stdin("test-password\n")
            .assert()
            .success();
    }
    cmd(&source)
        .args([
            "add",
            "--issuer",
            "Example",
            "--label",
            "alice",
            "--secret-stdin",
        ])
        .write_stdin("test-password\nJBSWY3DP\n")
        .assert()
        .success();
    let before = std::fs::read(&destination).unwrap();
    cmd(&destination)
        .arg("merge")
        .arg(&source)
        .args(["--source-password-stdin", "--dry-run", "--json"])
        .write_stdin("test-password\ntest-password\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"added\":1"));
    assert_eq!(before, std::fs::read(&destination).unwrap());
    cmd(&destination)
        .arg("merge")
        .arg(&source)
        .arg("--source-password-stdin")
        .write_stdin("test-password\nwrong-password\n")
        .assert()
        .failure();
    assert_eq!(before, std::fs::read(&destination).unwrap());
    cmd(&destination)
        .arg("merge")
        .arg(&source)
        .arg("--source-password-stdin")
        .write_stdin("test-password\ntest-password\n")
        .assert()
        .success();
    cmd(&destination)
        .args(["list", "--metadata-only", "--json"])
        .write_stdin("test-password\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));
    cmd(&destination)
        .arg("merge")
        .arg(&source)
        .args(["--source-password-stdin", "--json"])
        .write_stdin("test-password\ntest-password\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"added\":0"));
}
