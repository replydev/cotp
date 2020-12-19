use std::{env};
use std::fs::{read_to_string,File};
use std::io::prelude::*;
//mod encryption;
mod database_loader;
use database_loader::OTPElement;
extern crate directories;
extern crate otp;
use otp::make_totp;
mod utils;
use utils::get_db_path;
mod cryptograpy;
fn main() {
    let version = "0.0.6";
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
    if args.len() == 1 {
        return false;
    }

    match &args[1][..]{
        "--import" =>{
            import_database(&args[2]);
            return true;
        },
        "--help" =>{
            println!("Help");
            return true;
        },
        "--add" =>{
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
        },
        "--remove" =>{
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
        },
        "--modify" =>{
            if args.len() == 6{
                let id = args[2].parse::<usize>().unwrap();
                let secret = &args[3];
                let issuer = &args[4];
                let label = &args[5];
                database_loader::modify_element(id, &secret, &issuer, &label).expect("An error occured");
            }
            else{
                println!("Invalid arguments, type cotp --modify <id> <secret> <issuer> <label>\nReplace the attribute value with \".\" to skip the attribute modification");
            }
            return true;
        },
        "--export" =>{
            if args.len() == 2{
                let export_result = database_loader::export_database();
                match export_result{
                    Ok(export_result) => {
                        println!("Database was successfully exported at {}", export_result);
                    },
                    Err(e) =>{
                        println!("An error as occured while exporting database: {}", e);
                    }
                }
            }
            else{
                println!("Invalid argument, type cotp --export");
            }
            return true;
        },
        _=>{
            println!("Invalid argument: {}, type cotp --help to get command options", args[1]);
            return true;
        }
    }
}

fn import_database(filename: &String){
    let mut unencrypted_content = read_to_string(filename).unwrap();
    let encrypted_content = cryptograpy::encrypt_string(&mut unencrypted_content,&cryptograpy::prompt_for_passwords("Insert password for database encryption: "));
    let mut encrypted_file = File::create(&get_db_path()).expect("Cannot create encrypted database file");
    encrypted_file.write_all(encrypted_content.as_bytes()).expect("Cannot write to encrypted file");
    println!("Successfully imported database");
}
