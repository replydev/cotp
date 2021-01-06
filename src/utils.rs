use std::fs::{File};
use std::io::prelude::*;
use std::path::{Path,PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::home_dir;
use super::database_loader;

#[cfg(debug_assertions)]
pub fn get_db_path() -> PathBuf{
    PathBuf::from("db.cotp")
}

#[cfg(not(debug_assertions))]
pub fn get_db_path() -> PathBuf{
    let mut home_dir = get_home_folder().to_str().unwrap().to_string();
    home_dir.push_str("/.cotp/db.cotp");
    PathBuf::from(home_dir)
}

pub fn get_home_folder() -> PathBuf {
    home_dir().unwrap()
}

pub fn create_db_if_needed(){
    if !Path::new(&get_db_path()).exists() {
        /*let mut database_file = File::create(&get_db_path()).expect("Cannot create encrypted database file");
        let encrypted_content = cryptograpy::encrypt_string(&mut String::from("[]"), &cryptograpy::prompt_for_passwords("Insert password for database encryption: "));
        write_to_file(&encrypted_content,&mut database_file);*/
        database_loader::overwrite_database_json("[]");
    }
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