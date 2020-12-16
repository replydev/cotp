use directories::BaseDirs;
use std::fs::{File,remove_file};
use std::path::Path;
use std::io::prelude::*;
use super::cryptograpy;

#[cfg(debug_assertions)]
pub fn get_db_path() -> String{
    String::from("db.cotp")
}

#[cfg(not(debug_assertions))]
pub fn get_db_path() -> String{
    let mut home_dir = get_home_folder();
    home_dir.push_str("/.cotp");
    home_dir.push_str("/db.cotp");
    home_dir
}

#[cfg(debug_assertions)]
pub fn get_unencrypted_db_path() -> String{
    String::from("db.cotp.plain")
}

#[cfg(not(debug_assertions))]
pub fn get_unencrypted_db_path() -> String{
    let mut home_dir = get_home_folder();
    home_dir.push_str("/.cotp");
    home_dir.push_str("/db.cotp.plain");
    home_dir
}

fn get_home_folder() -> String {
    let base_dirs = BaseDirs::new().unwrap();
    let home = base_dirs.home_dir().to_str().unwrap();
    home.to_string()
}

pub fn create_db_if_needed(){
    if !Path::new(&get_db_path()).exists() {
        create_unencrypted_file();
        cryptograpy::encrypt(&mut File::open(&get_unencrypted_db_path()).expect("Cannot open unencrypted file"), &mut File::create(&get_db_path()).expect("Failed to create file"), &cryptograpy::prompt_for_passwords("Insert password for encrypt: ")).expect("Failed to encrypt");
        remove_file(&get_unencrypted_db_path()).expect("Failed to remove unencrypted_file");
    }
}

pub fn write_to_file(content: &str, file: &mut File){
    file.write_all(content.as_bytes()).expect("Error writing to file");
    file.sync_all().expect("Sync failed");
}

fn create_unencrypted_file(){
    let mut unencrypted_file = File::create(&get_unencrypted_db_path()).expect("Failed to create db");
    write_to_file(&get_example_content(),&mut unencrypted_file);
}

fn get_example_content() -> String{
    String::from("[]")
}