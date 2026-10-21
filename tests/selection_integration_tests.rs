use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

fn command(path: &std::path::Path) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("cotp");
    command
        .args(["--password-stdin", "--database-path"])
        .arg(path);
    command
}

#[test]
fn account_identity_survives_edit_and_selectors_do_not_choose_ambiguous_accounts() {
    let dir = assert_fs::TempDir::new().unwrap();
    let path = dir.path().join("accounts.cotp");
    std::fs::copy("test_samples/cli_integration_test/empty_database", &path).unwrap();
    for (label, alias, counter) in [("alice", "work", "0"), ("bob", "personal", "1")] {
        command(&path)
            .args([
                "add",
                "--issuer",
                "Example",
                "--label",
                label,
                "--alias",
                alias,
                "--type",
                "hotp",
                "--counter",
                counter,
                "--secret-stdin",
            ])
            .write_stdin("12345678\nGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ\n")
            .assert()
            .success();
    }
    let output = command(&path)
        .args(["list", "--json"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let id = list
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["alias"] == "work")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(id.len(), 32);
    command(&path)
        .args(["extract", "--alias", "WORK"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .stdout(contains("755224"));
    command(&path)
        .args(["extract", "--issuer", "example"])
        .write_stdin("12345678\n")
        .assert()
        .failure()
        .stderr(contains("Ambiguous"));
    command(&path)
        .args(["extract", "--index", "0"])
        .write_stdin("12345678\n")
        .assert()
        .failure()
        .stderr(contains("start at 1"));
    command(&path)
        .args([
            "edit",
            "--id",
            &id,
            "--issuer",
            "Renamed",
            "--set-alias",
            "primary",
        ])
        .write_stdin("12345678\n")
        .assert()
        .success();
    let output = command(&path)
        .args(["list", "--json"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let account = list
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == id)
        .unwrap();
    assert_eq!(account["issuer"], "Renamed");
    assert_eq!(account["alias"], "primary");
    command(&path)
        .args(["delete", "--issuer", "*"])
        .write_stdin("12345678\n")
        .assert()
        .failure()
        .stderr(contains("Ambiguous"));
}
