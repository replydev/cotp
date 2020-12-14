use std::env;
use std::fs;
//mod encryption;
mod database_loader;
extern crate directories;
extern crate otp;
use otp::make_totp;
mod utils;
use utils::get_db_path;
fn main() {
    let version = "0.0.2";
    println!("cotp v{}",version);
    println!("written by @replydev\n");

    let args: Vec<String> = env::args().collect();

    if !args_parser(args){
        show_codes();
    }
}

fn show_codes(){
    let elements: Vec<database_loader::OTPElement> = database_loader::get_elements();
    for i in 0..elements.len() {
        let secret : &str = &elements[i].secret(); //we have replaced '=' in this method
        println!("{}) - {}: {}",i+1,elements[i].label(),make_totp(secret,30, 0).unwrap());
    }
}

fn args_parser(args: Vec<String>) -> bool {
    if args.len() >= 2{
        if args[1] == "--import"{
            import_database(&args[2]);
            return true;
        }
        else if args[1] == "--help"{
            println!("Help");
            return true;
        }
        else if args[1] == "--add"{
            println!("To be implemented");
            return true;
        }
        else if args[1] == "--remove"{
            if args.len() == 3{
                let id = args[2].parse::<usize>().unwrap();
                if database_loader::remove_element_from_db(id) {
                    println!("ok");
                }
                else{
                    println!("{} is a wrong index", id);
                }
            }
            else{
                println!("Invalid argument, type cotp --remove <index>");
            }
            return true;
        }
        else{
            println!("Invalid argument: {}, type cotp --help to get command options", args[1]);
            return true;
        }
    }
    else{
        false
    }
}

fn import_database(filename: &String){
    fs::copy(filename,&get_db_path()).expect("Failed to import database");
    println!("Successfully imported database");
}
