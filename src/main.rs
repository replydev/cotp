use std::env;
mod database_loader;
mod utils;
mod argument_functions;
mod otp_helper;
mod cryptograpy;
mod importers;
mod otp;
use sodiumoxide;
use std::thread::sleep;
use std::time::Duration;

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

fn init() -> Result<(), ()>{
    match sodiumoxide::init(){
        Err(()) => Err(()),
        _=> {
            utils::create_db_if_needed();
            Ok(())
        },
    }
}

fn main() {
    let version = "0.1.1";
    print_title(version);
    let init_result = init();
    match init_result {
        Ok(()) => {},
        Err(()) => { 
            println!("Failed to init cotp");
            return;
        }
    }
    let args: Vec<String> = env::args().collect();
    if !args_parser(args){
        utils::create_db_if_needed();
        dashboard();
    }
}

fn dashboard(){
    let mut lines;
    match otp_helper::read_codes(){
        Ok(elements) => {
            if elements.len() == 0{
                println!("No codes, type \"cotp -h\" to get help");
            }
            else{
                loop{
                    utils::print_progress_bar();
                    lines = otp_helper::show_codes(&elements);
                    sleep(Duration::from_millis(1000));
                    print!("\x1B[{}A", lines + 1);
                }
            }
        },
        Err(e) => println!("An error as occurred: {}",e),
    }
}

fn args_parser(args: Vec<String>) -> bool {
    if args.len() == 1 {
        return false;
    }

    match &args[1][..]{
        "-i" | "--import" => argument_functions::import(args),
        "-h" | "--help" => argument_functions::help(),
        "-a" | "--add" => argument_functions::add(args),
        "-r" | "--remove" => argument_functions::remove(args),
        "-e" | "--edit" => argument_functions::edit(args),
        "-ex"| "--export" => argument_functions::export(args),
        "-j" | "--json" => argument_functions::json(args),
        "-s" | "--single" => argument_functions::single(args),
        _=>{
            println!("Invalid argument: {}, type cotp -h to get command options", args[1]);
            return true;
        }
    }
    true
}
