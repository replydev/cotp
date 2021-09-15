use std::fs::{File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::home_dir;
use crate::otp::otp_element::OTPElement;

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

#[cfg(debug_assertions)]
fn get_cotp_folder() -> PathBuf{
    PathBuf::from(".")
}

// Pushing an absolute path to a PathBuf replaces the entire PathBuf: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push
#[cfg(not(debug_assertions))]
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

pub fn delete_db() -> std::io::Result<()> {
    std::fs::remove_file(get_db_path())
}

pub fn write_to_file(content: &str, file: &mut File) -> Result<(),std::io::Error>{
    file.write_all(content.as_bytes())?;
    file.sync_all()
}

pub fn print_progress_bar(width: u64){
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let in_ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1000000;
    let step = in_ms % 30000;
    let idx = step * width / 30000;
    //println!("[{progress:>w$}]",  progress="=".repeat(idx as usize),w=width as usize,);
    println!("[{progress:<w$}]",  progress="=".repeat(idx as usize),w=width as usize,);
}

pub fn clear_lines(lines: usize,exit: bool){
    if exit{
        //  does not work during ctrl clear
        print!("\x1B[{}A\x1B[0G\x1B[0J", lines);
    }
    else{
        print!("\x1B[{}A", lines);
    }
}

pub fn check_elements(id: usize,elements: &Vec<OTPElement>) -> Result<(),String>{
    if elements.len() == 0{
        return Err(String::from("there are no elements in your database. Type \"cotp -h\" to get help."));
    }

    if id >= elements.len(){
        return Err(format!("{} is a bad index",id+1));
    }

    Ok(())
}


#[cfg(test)]
mod tests{
    use super::create_db_if_needed;
    #[test]
    fn test_db_creation() {
        assert_eq!(Ok(true),create_db_if_needed());
    }
}