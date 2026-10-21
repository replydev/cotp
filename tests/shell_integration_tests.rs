use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn completions_and_manual_do_not_open_a_vault() {
    let dir = assert_fs::TempDir::new().unwrap();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        Command::new(assert_cmd::cargo::cargo_bin!("cotp"))
            .current_dir(dir.path())
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains("cotp"));
    }
    Command::new(assert_cmd::cargo::cargo_bin!("cotp"))
        .current_dir(dir.path())
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH"));
    assert!(!dir.path().join("db.cotp").exists());
}

#[test]
fn explicit_unknown_profile_is_an_error() {
    let dir = assert_fs::TempDir::new().unwrap();
    let config = dir.path().join("config.json");
    std::fs::write(&config, "{}").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!("cotp"))
        .args([
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "missing",
            "list",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown vault profile"));
}
