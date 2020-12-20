use std::env;
mod database_loader;
extern crate directories;
mod utils;
mod argument_functions;
mod otp_helper;
mod cryptograpy;

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

fn main() {
    let version = "0.0.6";
    print_title(version);
    let args: Vec<String> = env::args().collect();
    if !args_parser(args){
        utils::create_db_if_needed();
        otp_helper::show_codes();
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
