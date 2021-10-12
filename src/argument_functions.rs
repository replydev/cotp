use crate::{cryptography, database_loader};
use crate::cryptography::prompt_for_passwords;
use crate::importers;
use crate::otp::{otp_element::OTPElement, otp_helper};
use zeroize::Zeroize;

pub fn help() {
    println!("USAGE:");
    println!("  cotp [SUBCOMMAND]");
    println!();
    println!("ARGUMENTS:");
    println!("  -a,--add [ISSUER] [LABEL] [ALGORITHM] [DIGITS]       | Add a new OTP code");
    println!("  -e,--edit [ID] [ISSUER] [LABEL] [ALGORITHM] [DIGITS] | Edit an OTP code");
    println!("  -r,--remove [ID]                                     | Remove an OTP code");
    println!("  -i,--import [APPNAME] [PATH]                         | Import a backup from a given application");
    println!("  -ex,--export                                         | Export the entire database in a plaintext json format");
    println!("  -j,--json                                            | Print results in json format");
    println!("  -s,--single                                          | Print OTP codes in single mode");
    println!("  -in,--info [ID]                                      | Print info of chosen OTP code");
    println!("  -chpw,--change-password                              | Change the database password");
    println!("  -h,--help                                            | Print this help");
}

pub fn import(args: Vec<String>) {
    if args.len() == 4 {
        let result: Result<Vec<OTPElement>, String>;
        let elements: Vec<OTPElement>;

        match &args[2][..] {
            "cotp" | "andotp" => result = importers::and_otp::import(&args[3]),
            "aegis" => result = importers::aegis::import(&args[3]),
            "gauth" |
            "google_authenticator" |
            "authy" => result = importers::converted::import(&args[3]),
            _ => {
                println!("Invalid argument: {}", &args[2]);
                return;
            }
        }

        match result {
            Ok(result) => elements = result,
            Err(e) => {
                eprintln!("An error occurred: {}", e);
                return;
            }
        }

        let mut pw = cryptography::prompt_for_passwords("Choose a password: ", 8, true);
        match database_loader::overwrite_database(elements, &pw) {
            Ok(()) => {
                println!("Successfully imported database");
            }
            Err(e) => {
                eprintln!("An error occurred during database overwriting: {}", e);
            }
        }
        pw.zeroize();
    } else {
        println!("Invalid arguments, type cotp --import [APPNAME] [PATH]");
        println!("cotp can import backup from:");
        println!("\"cotp\"");
        println!("\"aegis\"");
        println!("\"andotp\"");
        println!("\"gauth\" or \"google_authenticator\"");
        println!("\"authy\"");
    }
}

pub fn add(args: Vec<String>) {
    if args.len() == 6 {
        let digits: u64 = match &args[5].parse::<>() {
            Ok(r) => *r,
            Err(_e) => 0,
        };
        if digits <= 0 {
            eprintln!("Insert a valid digits value!");
            return;
        }

        let mut secret = prompt_for_passwords("Insert the secret: ", 0, false);
        match database_loader::add_element(&secret, &args[2], &args[3], &args[4], digits) {
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("An error occurred: {}", e)
        }
        secret.zeroize();
    } else {
        println!("Invalid arguments, type cotp --add [ISSUER] [LABEL] [ALGORITHM] [DIGITS]");
    }
}

pub fn remove(args: Vec<String>) {
    if args.len() == 3 {
        let id = match args[2].parse::<usize>() {
            Ok(result) => result,
            Err(_) => {
                eprintln!("Invalid index: {}",args[2]);
                return;
            }
        };
        match database_loader::remove_element_from_db(id) {
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("An error has occurred: {}", e)
        }
    } else {
        println!("Invalid argument, type cotp --remove <index>");
    }
}

pub fn edit(args: Vec<String>) {
    if args.len() == 7 {
        let id = match args[2].parse::<usize>() {
            Ok(result) => result,
            Err(_) => {
                eprintln!("Invalid index: {}",args[2]);
                return;
            }
        };
        let mut secret = prompt_for_passwords("Insert the secret (type ENTER to skip modification): ", 0, false);
        let issuer = &args[3];
        let label = &args[4];
        let algorithm = &args[5];
        let digits: u64 = match &args[6].parse::<>() {
            Ok(r) => *r,
            Err(_e) => 0,
        };
        match database_loader::edit_element(id, &secret, &issuer, &label, &algorithm, digits) {
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("An error occurred: {}", e)
        }
        secret.zeroize();
    } else {
        println!("Invalid arguments, type cotp --edit [ID] [ISSUER] [LABEL] [ALGORITHM] [DIGITS]\n\nReplace the attribute value with \".\" to skip the attribute modification");
    }
}

pub fn export(args: Vec<String>) {
    if args.len() == 2 {
        let export_result = database_loader::export_database();
        match export_result {
            Ok(export_result) => {
                println!("Database was successfully exported at {}", export_result.to_str().unwrap_or("**Invalid path**"));
            }
            Err(e) => {
                eprintln!("An error occurred while exporting database: {}", e);
            }
        }
    } else {
        println!("Invalid argument, type cotp --export");
    }
}

pub fn json(args: Vec<String>) {
    if args.len() == 2 {
        match otp_helper::get_json_results() {
            Ok(results) => println!("{}", results),
            Err(e) => eprintln!("An error occurred while getting json result: {}", e),
        }
    } else {
        println!("Invalid argument, type cotp --json");
    }
}

pub fn single(args: Vec<String>) {
    if args.len() == 2 {
        match otp_helper::read_codes() {
            Ok(result) => {
                if result.len() == 0 {
                    println!("No codes, type \"cotp -h\" to get help");
                } else {
                    otp_helper::list_codes(&result);
                }
            }
            Err(e) => eprintln!("An error occurred: {}", e)
        }
    } else {
        println!("Invalid argument, type cotp --single");
    }
}

pub fn info(args: Vec<String>) {
    if args.len() == 3 {
        let id = match args[2].parse::<usize>(){
            Ok(result) => result,
            Err(_) => {
                eprintln!("Invalid index: {}",args[2]);
                return;
            }
        };
        match otp_helper::print_element_info(id) {
            Ok(()) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    } else {
        eprintln!("Invalid arguments, type cotp --info [ID]");
    }
}

pub fn change_password(args: Vec<String>) {
    if args.len() == 2 {
        let mut old_password = cryptography::prompt_for_passwords("Old password: ", 8, false);
        let decrypted_text = database_loader::read_decrypted_text(&old_password);
        old_password.zeroize();
        match decrypted_text {
            Ok(s) => {
                let mut new_password = cryptography::prompt_for_passwords("New password: ", 8, true);
                match database_loader::overwrite_database_json(&s, &new_password) {
                    Ok(()) => println!("Password changed"),
                    Err(e) => eprintln!("An error has occurred: {}", e),
                }
                new_password.zeroize();
            }
            Err(e) => {
                eprintln!("An error has occurred: {}", e);
            }
        }
    } else {
        println!("Invalid arguments, type cotp --change-password");
    }
} 