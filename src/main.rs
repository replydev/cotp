mod database_loader;
mod utils;
mod argument_functions;
mod cryptograpy;
mod importers;
mod otp;
mod print_settings;
use std::{env, io::stdout};
use cursor::MoveTo;
use sodiumoxide;
use std::thread::sleep;
use std::time::Duration;
use otp::otp_helper;
#[macro_use]
extern crate crossterm;
use crossterm::{cursor, event::{KeyCode,Event,read}, terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode}};
use std::thread;
use std::sync::{Arc, Mutex};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
fn print_title(){
    println!("cotp v{} **DEBUG VERSION**",VERSION);
    println!("written by @replydev\n");
}

#[cfg(not(debug_assertions))]
fn print_title(){
    println!("cotp v{}",VERSION);
    println!("written by @replydev\n");
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
                let current_page = Arc::new(Mutex::new(1));
                let current_page_clone = current_page.clone();
                let exit_flag = Arc::new(Mutex::new(false));
                let exit_flag_clone = exit_flag.clone();
                //let elements_mutex = Arc::new(Mutex::new(elements));
                //let elements_mutex_clone = elements_mutex.clone();
                enable_raw_mode().unwrap();
                let elements_len = elements.len();
                thread::spawn(move || {
                    loop {
                        let event = read().unwrap();
                        if event == Event::Key(KeyCode::Char('q').into()) {
                            *exit_flag_clone.lock().unwrap() = true;
                        }
                        if event == Event::Key(KeyCode::Right.into()) && *current_page.lock().unwrap() < utils::get_max_pages(elements_len, utils::get_usable_table_rows()){
                            *current_page.lock().unwrap() += 1;
                        }
                        if event == Event::Key(KeyCode::Left.into()) && *current_page.lock().unwrap() > 1 {
                            *current_page.lock().unwrap() -= 1;
                        }
                    }
                });
                while !*exit_flag.lock().unwrap(){
                    let width = otp_helper::show_codes(&elements,*current_page_clone.lock().unwrap());
                    utils::print_progress_bar(width as u64);
                    sleep(Duration::from_millis(500));
                }
                let mut stdout = stdout();
                execute!(&mut stdout,Clear(ClearType::All),MoveTo(0,0)).unwrap();
                disable_raw_mode().unwrap();
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
