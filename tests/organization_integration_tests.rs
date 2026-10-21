use assert_cmd::cargo::cargo_bin_cmd;
use std::path::Path;

fn command(path: &Path) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("cotp");
    command
        .args(["--no-input", "--password-stdin", "--database-path"])
        .arg(path);
    command
}

fn list(path: &Path, filters: &[&str]) -> serde_json::Value {
    let output = command(path)
        .args(["list", "--json", "--metadata-only"])
        .args(filters)
        .write_stdin("12345678\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn organization_persists_filters_and_orders_without_changing_account_identity() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    command(&path)
        .arg("init")
        .write_stdin("12345678\n")
        .assert()
        .success();
    for (label, extra) in [
        (
            "alice",
            vec![
                "--tag",
                "work",
                "--tag",
                "WORK",
                "--tag",
                "mail",
                "--favorite",
            ],
        ),
        ("bob", vec![]),
        ("carol", vec!["--order", "-1"]),
    ] {
        command(&path)
            .args([
                "add",
                "--issuer",
                "Example",
                "--label",
                label,
                "--alias",
                label,
                "--type",
                "hotp",
                "--counter",
                "0",
                "--secret-stdin",
            ])
            .args(extra)
            .write_stdin("12345678\nGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ\n")
            .assert()
            .success();
    }
    let all = list(&path, &[]);
    assert_eq!(all[0]["label"], "carol");
    assert_eq!(all[1]["label"], "alice");
    assert_eq!(all[2]["label"], "bob");
    let filtered = list(&path, &["--tag", "WORK", "--tag", "mail", "--favorite"]);
    assert_eq!(filtered.as_array().unwrap().len(), 1);
    assert_eq!(filtered[0]["index"], 2);
    assert_eq!(filtered[0]["tags"].as_array().unwrap().len(), 2);
    let id = filtered[0]["id"].as_str().unwrap();
    command(&path)
        .args(["code", "--tag", "work", "--favorite", "--preview"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .stdout("755224\n");
    command(&path)
        .args([
            "edit",
            "--tag",
            "WORK",
            "--favorite",
            "--add-tag",
            "bank",
            "--remove-tag",
            "MAIL",
            "--set-favorite",
            "false",
            "--order",
            "-2",
        ])
        .write_stdin("12345678\n")
        .assert()
        .success();
    let edited = list(&path, &[]);
    assert_eq!(edited[0]["id"], id);
    assert_eq!(edited[0]["favorite"], false);
    assert_eq!(edited[0]["tags"], serde_json::json!(["work", "bank"]));
    assert_eq!(list(&path, &["--favorite"]), serde_json::json!([]));
    command(&path)
        .args(["edit", "--id", id, "--clear-tags", "--clear-order"])
        .write_stdin("12345678\n")
        .assert()
        .success();
    let cleared = list(&path, &[]);
    assert_eq!(cleared[0]["label"], "carol");
    assert_eq!(cleared[1]["tags"], serde_json::json!([]));
    assert!(cleared[1]["order"].is_null());
    let before = std::fs::read(&path).unwrap();
    command(&path)
        .args(["edit", "--id", id, "--add-tag", " "])
        .write_stdin("12345678\n")
        .assert()
        .code(2);
    assert_eq!(std::fs::read(&path).unwrap(), before);
}
