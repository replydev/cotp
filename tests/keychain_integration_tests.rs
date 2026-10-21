use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

#[test]
fn no_input_remember_requires_explicit_authentication_without_accessing_native_store() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault.cotp");
    std::fs::copy("test_samples/cli_integration_test/empty_database", &vault).unwrap();
    let before = std::fs::read(&vault).unwrap();
    cargo_bin_cmd!("cotp")
        .arg("--database-path")
        .arg(&vault)
        .args(["--no-input", "keychain", "remember"])
        .assert()
        .failure()
        .stderr(contains("Interactive input is disabled"));
    assert_eq!(std::fs::read(vault).unwrap(), before);
}

#[test]
fn explicit_password_overrides_enabled_keychain_without_accessing_native_store() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault.cotp");
    let config = dir.path().join("config.json");
    std::fs::copy("test_samples/cli_integration_test/empty_database", &vault).unwrap();
    std::fs::write(&config, r#"{"keychain":true}"#).unwrap();
    cargo_bin_cmd!("cotp")
        .arg("--database-path")
        .arg(&vault)
        .arg("--config")
        .arg(&config)
        .args(["--no-input", "--password-stdin", "list", "--json"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .stdout("[]\n");
}
