use dirs::{data_dir, home_dir};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};

use crate::arguments::CotpArgs;

const CURRENT_DB_PATH: &str = "./db.cotp";
const XDG_PATH: &str = "cotp/db.cotp";
const HOME_PATH: &str = ".cotp/db.cotp";

pub static DATABASE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Initialize singleton database path
pub fn init_path(args: &CotpArgs) -> PathBuf {
    DATABASE_PATH
        .get_or_init(|| {
            args.database_path
                .as_ref()
                .map(String::from)
                .or(env::var("COTP_DB_PATH").ok())
                .map(PathBuf::from)
                .unwrap_or_else(|| get_default_db_path())
        })
        .to_owned()
}

// Pushing an absolute path to a PathBuf replaces the entire PathBuf: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push
fn get_default_db_path() -> PathBuf {
    // If db.cotp is present in the current directory or we are using a debug artifact, do not use the one in home dir
    // First condition is optimized away in release mode
    let portable_path: PathBuf = PathBuf::from(CURRENT_DB_PATH);
    if cfg!(debug_assertions) || portable_path.exists() {
        return portable_path;
    }

    let home_path = home_dir().map(|path| path.join(HOME_PATH));

    data_dir()
        .map(PathBuf::from)
        .map(|p| p.join(XDG_PATH))
        .map(|xdg| {
            if !xdg.exists() {
                if let Some(home) = &home_path {
                    if home.exists() {
                        fs::create_dir_all(xdg.parent().unwrap()).expect("Failed to create dir");
                        fs::copy(home, xdg.as_path())
                            .expect("Failed on copy from legacy dir to XDG_DATA_HOME");
                    }
                }
            }
            xdg
        })
        .or(home_path)
        .unwrap_or(portable_path)
}
