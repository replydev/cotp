use std::env;
use std::fs;
//mod encryption;
mod database_loader;
extern crate directories;
extern crate otp;
use otp::make_totp;
use directories::{BaseDirs, UserDirs, ProjectDirs};
fn main() {
    println!("cotp - written by @replydev");

    let args: Vec<String> = env::args().collect();

    if args.len() > 2{
        if args[1] == "--import"{
            import_database(&args[2]);
        }
        else{
            println!("Help")
        }
    }
    else{
        let elements: Vec<database_loader::OTPElement> = database_loader::read_from_file(get_db_path());
        for i in 0..elements.len() {

            let secret : &str = &elements[i].secret();

            println!("{}: {}",elements[i].label(),make_totp(secret,30, 0).unwrap());


            //println!("{} - {} - {}", elements[i].secret(), elements[i].label(), elements[i].algorithm());
        }
    }
}


fn get_db_path() -> String{
    let mut home_dir :String = "".to_string();

    if let Some(base_dirs) = BaseDirs::new(){
        if let Some(home) = base_dirs.home_dir().to_str() {
            home_dir = home.to_string();
        }
    }

    home_dir.push_str("/.cotp");

    fs::create_dir_all(&home_dir).expect("Failed to create directory!");
    home_dir.push_str("/db");
    home_dir
}

fn import_database(filename: &String){

    let home_dir = get_db_path();
    
    fs::copy(filename,&home_dir).expect("Failed to import database");
    println!("Successfully imported database");
}
