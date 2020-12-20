use std::env;
mod database_loader;
use database_loader::OTPElement;
extern crate directories;
extern crate otp;
use otp::make_totp;
mod utils;
mod argument_functions;
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

fn get_good_otp_code(element: &OTPElement) -> String {
    let otp = make_totp(
        &element.secret(), //we have replaced '=' in this method
               element.period(), 0).unwrap();
    let mut s_otp = otp.to_string();

    while s_otp.len() < element.digits() as usize {
        s_otp = String::from("0") + &s_otp;
    }
    s_otp
}

fn print_totp(i: usize,element: &OTPElement){
    if element.issuer() != ""{
        println!("{}) {} - {}: {}",i+1,element.issuer(),element.label(),get_good_otp_code(&element));
    }else{
        println!("{}) {}: {}",i+1,element.label(),get_good_otp_code(&element));
    }
}

fn args_parser(args: Vec<String>) -> bool {
    if args.len() == 1 {
        return false;
    }

    match &args[1][..]{
        "-i"  | "--import" =>{argument_functions::import(args);},
        "-h"  | "--help" =>{argument_functions::help();},
        "-a"  | "--add" =>{argument_functions::add(args);},
        "-r"  | "--remove" =>{argument_functions::remove(args);},
        "-e"  | "--edit" =>{argument_functions::edit(args);},
        "-ex" | "--export" =>{argument_functions::export(args);},
        _=>{
            println!("Invalid argument: {}, type cotp -h to get command options", args[1]);
            return true;
        }
    }
    true
}
