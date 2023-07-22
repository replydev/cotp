use std::env;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    if Ok("release".to_owned()) == env::var("PROFILE") {
        println!("cargo:rustc-env=COTP_VERSION={}", version);
    } else {
        // Suffix with -DEBUG
        println!("cargo:rustc-env=COTP_VERSION={}-DEBUG", version);
    }
}
