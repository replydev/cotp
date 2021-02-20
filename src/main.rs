mod database_loader;
mod utils;
mod argument_functions;
mod cryptograpy;
mod importers;
mod otp;
use std::env;
use sodiumoxide;
use std::thread::sleep;
use std::time::Duration;
use ctrlc;
use otp::otp_helper;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
fn print_title(){
    println!("cotp v{}",VERSION);
    println!("written by @replydev\n");
    println!("****DEBUG VERSION****\n");
}

#[cfg(not(debug_assertions))]
fn print_title(){
    println!("cotp v{}",VERSION);
    println!("written by @replydev\n");
}

#[cfg(debug_assertions)]
fn init_ctrlc_handler(lines: usize){
    ctrlc::set_handler(move || {
        utils::clear_lines(lines + 7,true);
        std::process::exit(0);
    }).expect("Failed to initialize ctrl-c handler");
}

#[cfg(not(debug_assertions))]
fn init_ctrlc_handler(lines: usize){
    ctrlc::set_handler(move || {
        utils::clear_lines(lines + 6,true);
        std::process::exit(0);
    }).expect("Failed to initialize ctrl-c handler");
}

fn init() -> Result<bool, String>{
    match sodiumoxide::init(){
        Err(()) => {
            return Err(String::from("Error during sodiumoxide initialization"))
        },
        _=> {},
    };
    match utils::create_db_if_needed() {
        Ok(value) => {
            if value {
                let pw = &cryptograpy::prompt_for_passwords("Choose a password: ", 8,true);
                match database_loader::overwrite_database_json("[]",pw){
                    Ok(()) => return Ok(true),
                    Err(_e) => return Err(String::from("An error occurred during database overwriting")),
                }
            }
            Ok(false)
        },
        Err(()) => {
            return Err(String::from("An error occurred during database creation"));
        },
    }
}

fn main() {
    print_title();
    let init_result = init();
    match init_result {
        Ok(true) => {
            println!("Database correctly initialized");
            return;
        },
        Ok(false) => {},
        Err(e) => { 
            println!("{}",e);
            return;
        }
    }
    let args: Vec<String> = env::args().collect();
    if !args_parser(args){
        dashboard();
    }
}

fn dashboard(){
    match otp_helper::read_codes(){
        Ok(elements) => {
            if elements.len() == 0{
                println!("No codes, type \"cotp -h\" to get help");
            }
            else{
                init_ctrlc_handler(elements.len());
                loop{
                    utils::print_progress_bar();
                    otp_helper::show_codes(&elements);
                    sleep(Duration::from_millis(1000));
                    utils::clear_lines(elements.len() * 2 + 4,false);
                }
            }
        },
        Err(e) => eprintln!("An error occurred: {}",e),
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
        "-in" | "--info" => argument_functions::info(args),
        "-chpw" | "--change-password" => argument_functions::change_password(args),
        _=>{
            println!("Invalid argument: {}, type cotp -h to get command options", args[1]);
            return true;
        }
    }
    true
}
