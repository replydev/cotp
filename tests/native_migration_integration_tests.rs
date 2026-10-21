use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn native_imports_and_selected_encrypted_aegis_export_work_without_password_arguments() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db.cotp");
    std::fs::copy("test_samples/cli_integration_test/empty_database", &db).unwrap();
    for source in [
        "twofas.2fas",
        "bitwarden.json",
        "onepassword.data",
        "ente.txt",
    ] {
        cargo_bin_cmd!("cotp")
            .args(["--password-stdin", "--database-path"])
            .arg(&db)
            .args(["import", "--yes"])
            .arg(format!("test_samples/migrations/{source}"))
            .write_stdin("12345678\n")
            .assert()
            .success();
    }
    let password_file = dir.path().join("password");
    std::fs::write(&password_file, "separate backup password\n").unwrap();
    let encrypted = dir.path().join("aegis.json");
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&db)
        .args([
            "export",
            "--aegis-encrypted",
            "--tag",
            "work",
            "--query",
            "example",
            "--export-password-file",
        ])
        .arg(&password_file)
        .arg("--path")
        .arg(&encrypted)
        .write_stdin("12345678\n")
        .assert()
        .success();
    assert!(
        !std::fs::read_to_string(&encrypted)
            .unwrap()
            .contains("JBSWY3DP")
    );
    let restored = dir.path().join("restored.cotp");
    std::fs::copy(
        "test_samples/cli_integration_test/empty_database",
        &restored,
    )
    .unwrap();
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&restored)
        .arg("import")
        .arg(&encrypted)
        .args(["--yes", "--import-password-file"])
        .arg(&password_file)
        .write_stdin("12345678\n")
        .assert()
        .success();
    let plaintext = dir.path().join("restored.json");
    cargo_bin_cmd!("cotp")
        .args(["--password-stdin", "--database-path"])
        .arg(&restored)
        .args(["export", "--plaintext", "--path"])
        .arg(&plaintext)
        .write_stdin("12345678\n")
        .assert()
        .success();
    let data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(plaintext).unwrap()).unwrap();
    let elements = data["elements"].as_array().unwrap();
    assert_eq!(elements.len(), 2);
    assert!(
        elements
            .iter()
            .all(|e| e["tags"] == serde_json::json!(["work"]))
    );
    assert!(elements.iter().any(|e| e["counter"] == 42));
}
