use std::fs::{File};
use std::io::prelude::*;
use std::path::{Path,PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::home_dir;
use crate::database_loader;

#[cfg(debug_assertions)]
pub fn get_db_path() -> PathBuf{
    PathBuf::from("db.cotp")
}

#[cfg(not(debug_assertions))]
pub fn get_db_path() -> PathBuf{
    let cotp_folder = get_cotp_folder();
    cotp_folder.join("db.cotp")
}

pub fn get_home_folder() -> PathBuf {
    home_dir().unwrap()
}
// Push an absolute path to a PathBuf replaces the entire PathBuf: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push
fn get_cotp_folder() -> PathBuf{
    let mut cotp_dir = PathBuf::new();
    cotp_dir.push(get_home_folder());
    cotp_dir.join(".cotp")
}

pub fn create_db_if_needed() -> Result<(),()>{
    if !get_cotp_folder().exists(){
        match std::fs::create_dir(get_cotp_folder()){
            Ok(()) => println!("Created .cotp folder"),
            Err(_e) => (),
        }
    }
    if !Path::new(&get_db_path()).exists() {
        database_loader::overwrite_database_json("[]");
    }
    Ok(())
}

pub fn write_to_file(content: &str, file: &mut File){
    file.write_all(content.as_bytes()).expect("Error writing to file");
    file.sync_all().expect("Sync failed");
}

pub fn print_progress_bar(){
    let width = 60;
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let in_ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1000000;
    let step = in_ms % 30000;
    let idx = step * width / 30000;
    println!("[{:60}]", "=".repeat(idx as usize));
}