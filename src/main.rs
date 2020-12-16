use std::{env, path::Path};
use std::fs;
//mod encryption;
mod database_loader;
use database_loader::OTPElement;
extern crate directories;
extern crate otp;
use otp::make_totp;
mod utils;
use utils::{get_db_path,get_unencrypted_db_path};
mod cryptograpy;
fn main() {
    let version = "0.0.4";
    print_title(version);
    let args: Vec<String> = env::args().collect();
    if !args_parser(args){
        utils::create_db_if_needed();
        show_codes();
    }
}

#[cfg(debug_assertions)]
fn print_title(version: &str){
    println!("cotp v{}",version);
    println!("written by @replydev\n");
    println!("****DEBUG VERSION****\n");
}

#[cfg(not(debug_assertions))]
fn print_title(version: &str){
    println!("cotp v{}",version);
    println!("written by @replydev\n");
}

fn show_codes(){
    let elements: Vec<database_loader::OTPElement> = database_loader::read_from_file();
    for i in 0..elements.len() {
        print_totp(i,&elements[i]);
    }
}

fn print_totp(i: usize,element: &OTPElement){
    if element.issuer() != ""{
        println!("{}) {} - {}: {}",i+1,element.issuer(),element.label(),make_totp(
            &element.secret(), //we have replaced '=' in this method
                   element.period(), 0).unwrap());
    }else{
        println!("{}) {}: {}",i+1,element.label(),make_totp(
            &element.secret(), //we have replaced '=' in this method
                   element.period(), 0).unwrap());
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
            if args.len() == 5{
                if database_loader::add_element(&args[2],&args[3],&args[4]){
                    println!("Success");
                }
                else{
                    println!("Invalid values");
                }
            }
            else{
                println!("Invalid arguments, type cotp --add <secret> <issuer> <label>");
            }
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
    fs::copy(filename,&get_unencrypted_db_path()).expect("Failed to import database");
    cryptograpy::encrypt(&mut fs::File::open(&get_unencrypted_db_path()).expect("Failed to encrypt file"), &mut fs::File::create(&get_db_path()).expect("Cannot create encrypted file"), &cryptograpy::prompt_for_passwords("Insert password for database encryption: ")).expect("Cannot decrypt encrypted database");
    fs::remove_file(Path::new(&get_unencrypted_db_path())).expect("Cannot delete unencrypted database");
    println!("Successfully imported database");
}
