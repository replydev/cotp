use std::fs::{File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::home_dir;
use terminal_size::{Width,Height,terminal_size};
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

// Push an absolute path to a PathBuf replaces the entire PathBuf: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push
#[cfg(not(debug_assertions))]
fn get_cotp_folder() -> PathBuf{
    let mut cotp_dir = PathBuf::new();
    cotp_dir.push(get_home_folder());
    cotp_dir.join(".cotp")
}

#[cfg(debug_assertions)]
pub fn create_db_if_needed() -> Result<bool,()>{
    let db_path = get_db_path();
    if !db_path.exists() {
        match std::fs::File::create(db_path){
            Ok(_f) => return Ok(true),
            Err(_e) => return Err(()),
        }
    }
    Ok(false)
}

#[cfg(not(debug_assertions))]
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

pub fn print_progress_bar(width: u64){
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let in_ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1000000;
    let step = in_ms % 30000;
    let idx = step * width / 30000;
    //println!("[{progress:>w$}]",  progress="=".repeat(idx as usize),w=width as usize,);
    println!("[{progress:<w$}]",  progress="=".repeat(idx as usize),w=width as usize,);
}

pub fn pow(base: f64,exp: i64) -> f64{
    if exp == 0{
        return 1.0;
    }
    if exp < 0{
        return 1.0 / pow(base,-1 * exp);
    }
    let mut tot = 1.0;
    for _i in 0..exp{
        tot *= base;
    }
    tot
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

pub fn get_terminal_height() -> usize {
    let size = terminal_size();
    if let Some((Width(_w), Height(h))) = size {
        return h as usize;
    } else {
        0
    }
}

pub fn get_usable_table_rows() -> usize {
    let terminal_height = get_terminal_height();
    if terminal_height <= 3  {
        panic!("Not enough terminal space!");
    }
    return get_terminal_height() - 4; // minus one becouse we have to count the last row
}

pub fn get_max_pages(elements: usize,usable: usize) -> usize{
    (elements as f32 / usable as f32).ceil() as usize
}

#[cfg(test)]
mod tests{
    use super::create_db_if_needed;
    use super::pow;
    #[test]
    fn test_db_creation() {
        assert_eq!(Ok(true),create_db_if_needed());
    }

    #[test]
    fn test_pow(){
        assert_eq!(64.0,pow(8.0,2));
        assert_eq!(1.0,pow(134234.0,0));
        assert_eq!(0.2,pow(5.0,-1));
        assert_eq!(0.000244140625,pow(64.0,-2));
    }
}