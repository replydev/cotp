mod database_loader;
mod utils;
mod argument_functions;
mod cryptograpy;
mod importers;
mod otp;
mod print_settings;
use std::env;
use sodiumoxide;
use utils::clear_lines;
use std::thread::sleep;
use std::time::Duration;
use ctrlc;
use otp::otp_helper;
use crossterm::event::{KeyCode,Event,read};
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

fn exit_clean(lines: usize){
    utils::clear_lines(lines + 3,true);
        std::process::exit(0);
}

fn init_ctrlc_handler(lines: usize){
    ctrlc::set_handler(move || {
        exit_clean(lines);
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
    clear_lines(utils::get_terminal_height(), true);
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
                let modified = Arc::new(Mutex::new(false));
                let modified_clone = modified.clone();
                //let elements_mutex = Arc::new(Mutex::new(elements));
                //let elements_mutex_clone = elements_mutex.clone();

                let elements_len = elements.len();
                init_ctrlc_handler(elements_len);
                thread::spawn(move || {
                    loop {
                        let event = read().unwrap();
                        *modified.lock().unwrap() = false;
                        if event == Event::Key(KeyCode::Char('q').into()) {
                            exit_clean(elements_len)
                        }
                        if event == Event::Key(KeyCode::Right.into()) && *current_page.lock().unwrap() < utils::get_max_pages(elements_len, utils::get_usable_table_rows()){
                            *current_page.lock().unwrap() += 1;
                            *modified.lock().unwrap() = true;
                        }
                        if event == Event::Key(KeyCode::Left.into()) && *current_page.lock().unwrap() > 1 {
                            *current_page.lock().unwrap() -= 1;
                            *modified.lock().unwrap() = true;
                        }
                    }
                });
                clear_lines(4, true);
                loop{
                    let terminal_height_before = utils::get_terminal_height();
                    let width = otp_helper::show_codes(&elements,*current_page_clone.lock().unwrap());
                    utils::print_progress_bar(width as u64);
                    sleep(Duration::from_millis(200));
                    let terminal_height_after = utils::get_terminal_height();
                    utils::clear_lines(elements_len + 3,(terminal_height_before != terminal_height_after) || *modified_clone.lock().unwrap());
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
