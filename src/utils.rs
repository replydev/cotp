use directories::BaseDirs;
use std::fs;
use std::io::prelude::*;

#[cfg(debug_assertions)]
pub fn get_db_path() -> String{
    String::from("db.cotp")
}

#[cfg(not(debug_assertions))]
pub fn get_db_path() -> String{
    let base_dirs = BaseDirs::new().unwrap();
    let home = base_dirs.home_dir().to_str().unwrap();
    let mut home_dir = home.to_string();
    home_dir.push_str("/.cotp");
    fs::create_dir_all(&home_dir).expect("Failed to create directory!");
    home_dir.push_str("/db.cotp");
    home_dir
}

pub fn write_to_file(content: &str, filename: &str){
    let mut file = fs::File::create(filename).unwrap();
    file.write_all(content.as_bytes()).expect("Error writing to file");
    file.sync_all().expect("Sync failed");
}