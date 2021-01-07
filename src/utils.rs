use std::fs::{File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::home_dir;

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

pub fn create_db_if_needed() -> Result<bool,()>{
    let cotp_folder = get_cotp_folder();
    if !cotp_folder.exists(){
        match std::fs::create_dir(cotp_folder){
            Ok(()) => {},
            Err(_e) => {},
        }
    }
    let db_path = get_db_path();
    if !db_path.exists() {
        match std::fs::File::create(db_path){
            Ok(_f) => return Ok(true),
            Err(_e) => return Err(()),
        }
    }
    Ok(false)
}

pub fn write_to_file(content: &str, file: &mut File) -> Result<(),std::io::Error>{
    file.write_all(content.as_bytes())?;
    file.sync_all()
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

#[cfg(test)]
mod tests{
    use super::create_db_if_needed;
    #[test]
    fn test_db_creation() {
        let result = create_db_if_needed();
        assert_eq!(Ok(()),result);
    }
}