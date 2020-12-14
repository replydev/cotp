use directories::BaseDirs;
use std::fs::{File};
use std::path::Path;
use std::io::prelude::*;

#[cfg(debug_assertions)]
pub fn get_db_path() -> String{
    String::from("db.cotp")
}

#[cfg(not(debug_assertions))]
pub fn get_db_path() -> String{
    let home_dir = get_home_folder();
    home_dir.push_str("/.cotp");
    home_dir.push_str("/db.cotp");
    create_db_if_needed(home_dir);
    home_dir
}

fn get_home_folder() -> String {
    let base_dirs = BaseDirs::new().unwrap();
    let home = base_dirs.home_dir().to_str().unwrap();
    home.to_string()
}

pub fn create_db_if_needed(){
    let path: &str = &get_db_path();
    if !Path::new(path).exists() {
        let mut file = File::create(path).expect("Failed to create db");
        file.write_all("[]".as_bytes()).expect("Failed to write to db");
    }
}

pub fn write_to_file(content: &str, filename: &str){
    let mut file = File::create(filename).unwrap();
    file.write_all(content.as_bytes()).expect("Error writing to file");
    file.sync_all().expect("Sync failed");
}