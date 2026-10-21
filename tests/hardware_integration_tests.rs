#[cfg(unix)]
mod unix {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn hardware_backend_does_not_open_vault_and_requires_one_match() {
        let dir = assert_fs::TempDir::new().unwrap();
        let executable = dir.path().join("ykman");
        std::fs::write(
            &executable,
            r#"#!/bin/sh
case "$*" in
  "oath accounts list") printf 'GitHub:personal\nGitHub:work\n' ;;
  "oath accounts code --single -- GitHub:work") printf '123456\n' ;;
  *) exit 8 ;;
esac
"#,
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let run = || {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cotp"));
            cmd.current_dir(dir.path()).env("PATH", dir.path());
            cmd
        };
        run()
            .args(["--backend", "yubikey", "list", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("GitHub:work"));
        run()
            .args(["hardware", "code", "github"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Ambiguous"));
        run()
            .args(["hardware", "code", "GitHub:work"])
            .assert()
            .success()
            .stdout("123456\n");
        assert!(!dir.path().join("db.cotp").exists());
    }
}
