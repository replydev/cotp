use std::{env, process::Command};

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    if Ok("release".to_owned()) == env::var("PROFILE") {
        println!("cargo:rustc-env=COTP_VERSION={version}");
    } else {
        // Suffix with -DEBUG
        // If we can get the last commit hash, let's append that also
        if let Some(last_commit) = get_last_commit() {
            println!("cargo:rustc-env=COTP_VERSION={version}-DEBUG-{last_commit}");
        } else {
            println!("cargo:rustc-env=COTP_VERSION={version}-DEBUG");
        }
    }
}

fn get_last_commit() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|e| e.status.success())
        .map(|e| String::from_utf8(e.stdout))
        .and_then(Result::ok)
}
