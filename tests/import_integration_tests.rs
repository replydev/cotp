use assert_cmd::cargo::cargo_bin_cmd;

fn database(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let path = dir.path().join("db.cotp");
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&path)
        .arg("init")
        .write_stdin("12345678\n")
        .assert()
        .success();
    path
}

#[test]
fn dry_run_and_rejected_entries_leave_vault_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let db = database(&dir);
    let before = std::fs::read(&db).unwrap();
    let input = dir.path().join("uris.txt");
    std::fs::write(&input, "otpauth://totp/Example:alice?secret=JBSWY3DP").unwrap();
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&db)
        .arg("import")
        .arg(&input)
        .arg("--dry-run")
        .write_stdin("12345678\n")
        .assert()
        .success();
    assert_eq!(std::fs::read(&db).unwrap(), before);
    std::fs::write(
        &input,
        "otpauth://totp/Example:alice?secret=JBSWY3DP\notpauth://totp/hidden?secret=INVALIDSECRET",
    )
    .unwrap();
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&db)
        .arg("import")
        .arg(&input)
        .write_stdin("12345678\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Import rejected"));
    assert_eq!(std::fs::read(&db).unwrap(), before);
}

#[test]
fn stdin_import_is_repeatable_and_plaintext_export_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let db = database(&dir);
    for _ in 0..2 {
        cargo_bin_cmd!("cotp")
            .args(["--password-stdin", "--database-path"])
            .arg(&db)
            .args(["import", "-", "--yes"])
            .write_stdin("12345678\notpauth://hotp/Example:alice?secret=JBSWY3DP&counter=42\n")
            .assert()
            .success();
    }
    let export = dir.path().join("export.json");
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&db)
        .args(["export", "--path"])
        .arg(&export)
        .write_stdin("12345678\n")
        .assert()
        .failure();
    assert!(!export.exists());
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&db)
        .args(["export", "--plaintext", "--path"])
        .arg(&export)
        .write_stdin("12345678\n")
        .assert()
        .success();
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(export).unwrap()).unwrap();
    assert_eq!(value["elements"].as_array().unwrap().len(), 1);
    assert_eq!(value["elements"][0]["counter"], 42);
}
