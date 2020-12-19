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
        "--import" =>{argument_functions::import(args);},
        "--help" =>{argument_functions::help();},
        "--add" =>{argument_functions::add(args);},
        "--remove" =>{argument_functions::remove(args);},
        "--modify" =>{argument_functions::modify(args);},
        "--export" =>{argument_functions::export(args);},
        _=>{
            println!("Invalid argument: {}, type cotp --help to get command options", args[1]);
            return true;
        }
    }
    true
}
