use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn lock_requests_do_not_unlock_or_modify_the_vault() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault.cotp");
    std::fs::write(&vault, b"opaque encrypted content").unwrap();
    for _ in 0..2 {
        Command::new(assert_cmd::cargo::cargo_bin!("cotp"))
            .args(["--database-path", vault.to_str().unwrap(), "lock"])
            .assert()
            .success()
            .stderr(predicate::str::contains("Lock requested"));
        assert_eq!(std::fs::read(&vault).unwrap(), b"opaque encrypted content");
        assert_eq!(
            std::fs::read(dir.path().join("vault.cotp.lock-request"))
                .unwrap()
                .len(),
            16
        );
    }
}

#[test]
fn clipboard_helper_does_not_open_a_vault() {
    let dir = tempfile::tempdir().unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!("cotp"))
        .current_dir(dir.path())
        .args(["--internal-clipboard-expiry", "unsupported", "0"])
        .write_stdin("sensitive content supplied only over stdin")
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert!(!dir.path().join("db.cotp").exists());
}
