use std::{fs::File, io::Write, path::PathBuf};

use serde::Serialize;
use zeroize::Zeroize;

pub mod andotp;

pub fn do_export<T>(to_be_saved: &T, exported_path: PathBuf) -> Result<PathBuf, String>
where
    T: ?Sized + Serialize,
{
    match serde_json::to_string(to_be_saved) {
        Ok(mut contents) => {
            if contents == "[]" {
                return Err("No contents to export, skipping...".to_owned());
            }
            let mut file = File::create(&exported_path).expect("Cannot create file");
            let contents_bytes = contents.as_bytes();
            file.write_all(contents_bytes)
                .expect("Failed to write contents");
            contents.zeroize();
            Ok(exported_path)
        }
        Err(e) => Err(format!("{e:?}")),
    }
}
