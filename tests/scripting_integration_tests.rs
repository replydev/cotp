use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::{contains, is_empty};

fn command(path: &std::path::Path) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("cotp");
    command
        .args(["--no-input", "--password-stdin", "--database-path"])
        .arg(path);
    command
}

fn init(path: &std::path::Path) {
    command(path)
        .arg("init")
        .write_stdin("12345678\n")
        .assert()
        .success()
        .stdout(is_empty());
}

#[test]
fn hotp_is_committed_before_output_and_preview_and_list_are_read_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    init(&path);
    command(&path)
        .args([
            "add",
            "--label",
            "alice",
            "--alias",
            "work",
            "--type",
            "hotp",
            "--counter",
            "0",
            "--secret-stdin",
        ])
        .write_stdin("12345678\nGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ\n")
        .assert()
        .success()
        .stdout(is_empty());
    let before = std::fs::read(&path).unwrap();
    for args in [vec!["code", "work", "--preview"], vec!["list", "--json"]] {
        command(&path)
            .args(args)
            .write_stdin("12345678\n")
            .assert()
            .success()
            .stdout(contains("755224"));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    let output = command(&path)
        .args(["code", "work", "--json"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["code"], "755224");
    assert_eq!(value["id"].as_str().unwrap().len(), 32);
    assert!(value["expires_at"].is_null());
    assert!(value["remaining"].is_null());
    command(&path)
        .args(["code", "work", "--preview"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .stdout("287082\n");
    let metadata = command(&path)
        .args(["list", "--json", "--metadata-only"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let metadata: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
    assert!(metadata[0].get("otp_code").is_none());
    command(&path)
        .args(["code", "work", "--watch"])
        .write_stdin("12345678\n")
        .assert()
        .code(3)
        .stdout(is_empty());
}

#[test]
fn no_input_guards_every_command_prompt_and_errors_keep_stdout_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    init(&path);
    command(&path)
        .args(["add", "--label", "alice", "--secret-stdin"])
        .write_stdin("12345678\nAA\n")
        .assert()
        .success();
    for args in [
        vec!["add", "--label", "bob"],
        vec!["delete", "alice"],
        vec!["passwd"],
    ] {
        command(&path)
            .args(args)
            .write_stdin("12345678\n")
            .assert()
            .code(3)
            .stdout(is_empty())
            .stderr(contains("Interactive input is disabled"));
    }
    command(&path)
        .args(["code", "missing"])
        .write_stdin("12345678\n")
        .assert()
        .code(3)
        .stdout(is_empty())
        .stderr(contains("No account"));
    command(&path)
        .args(["list"])
        .write_stdin("wrong-password\n")
        .assert()
        .code(1)
        .stdout(is_empty());
    command(&path)
        .arg("init")
        .write_stdin("12345678\n")
        .assert()
        .code(1)
        .stdout(is_empty());
    cargo_bin_cmd!("cotp")
        .args(["--no-input", "--database-path"])
        .arg(&path)
        .args(["backup", "verify", "unused.cotp-backup"])
        .assert()
        .code(1)
        .stdout(is_empty())
        .stderr(contains("Interactive input is disabled"));
    cargo_bin_cmd!("cotp")
        .args(["--no-input", "--database-path"])
        .arg(&path)
        .arg("list")
        .assert()
        .code(1)
        .stdout(is_empty())
        .stderr(contains("Interactive input is disabled"));
}

#[cfg(unix)]
#[test]
fn password_descriptor_preserves_stdin_for_import_and_new_password() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    init(&path);
    let password = dir.path().join("password");
    std::fs::write(&password, "12345678\n").unwrap();
    let mut shell = assert_cmd::Command::new("sh");
    shell
        .args(["-c", "exec 3<\"$1\"; shift; exec \"$@\"", "sh"])
        .arg(&password)
        .arg(assert_cmd::cargo::cargo_bin!("cotp"))
        .args(["--no-input", "--password-fd", "3", "--database-path"])
        .arg(&path)
        .args(["import", "-", "--yes"])
        .write_stdin("otpauth://totp/Test:alice?secret=JBSWY3DP\n")
        .assert()
        .success()
        .stdout(is_empty());
    let before = std::fs::read(&path).unwrap();
    command(&path)
        .args(["code", "alice", "--min-validity", "30s"])
        .write_stdin("12345678\n")
        .assert()
        .code(3)
        .stdout(is_empty())
        .stderr(contains("shorter"));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let output = command(&path)
        .args(["code", "alice", "--json", "--min-validity", "1s"])
        .write_stdin("12345678\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["code"].as_str().unwrap().len(), 6);
    assert!(value["remaining"].as_u64().unwrap() >= 1);
    assert_eq!(value["expires_at"].as_u64().unwrap() % 30, 0);
    let mut shell = assert_cmd::Command::new("sh");
    shell
        .args(["-c", "exec 3<\"$1\"; shift; exec \"$@\"", "sh"])
        .arg(&password)
        .arg(assert_cmd::cargo::cargo_bin!("cotp"))
        .args(["--no-input", "--password-fd", "3", "--database-path"])
        .arg(&path)
        .args(["passwd", "--new-password-fd", "0"])
        .write_stdin("new-password\n")
        .assert()
        .success();
    command(&path)
        .args(["list", "--metadata-only"])
        .write_stdin("new-password\n")
        .assert()
        .success();
}

#[test]
fn watch_flushes_one_json_line_per_interval() {
    use std::{
        io::{BufRead, BufReader, Write},
        process::{Command, Stdio},
        sync::mpsc,
        time::Duration,
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    init(&path);
    command(&path)
        .args(["add", "--label", "watch", "--period", "1", "--secret-stdin"])
        .write_stdin("12345678\nAA\n")
        .assert()
        .success();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("cotp"))
        .args(["--no-input", "--password-stdin", "--database-path"])
        .arg(&path)
        .args(["code", "watch", "--watch", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"12345678\n")
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().take(2) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let first = receiver.recv_timeout(Duration::from_secs(10));
    let second = receiver.recv_timeout(Duration::from_secs(3));
    child.kill().unwrap();
    child.wait().unwrap();
    reader.join().unwrap();
    let first: serde_json::Value = serde_json::from_str(&first.unwrap().unwrap()).unwrap();
    let second: serde_json::Value = serde_json::from_str(&second.unwrap().unwrap()).unwrap();
    assert_eq!(
        second["expires_at"].as_u64().unwrap(),
        first["expires_at"].as_u64().unwrap() + 1
    );
}
