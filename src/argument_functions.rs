use crate::{cryptography, database_management};
use crate::cryptography::prompt_for_passwords;
use crate::importers;
use crate::otp::otp_helper;
use clap::ArgMatches;
use zeroize::Zeroize;

pub fn import(matches: &ArgMatches) {
    let app_name = matches.value_of("appname").unwrap();
    let path = matches.value_of("path").unwrap();

    let result = match app_name {
        "cotp" | "andotp" => importers::and_otp::import(path),
        "aegis" => importers::aegis::import(path),
        "gauth" | "authy" | "mauth" => importers::converted::import(path),
        _ => {
            println!("Invalid argument: {}", app_name);
            return;
        }
    };

    let elements = match result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            return;
        }
    };

    let mut pw = cryptography::prompt_for_passwords("Choose a password: ", 8, true);
    match database_management::overwrite_database(elements, &pw) {
        Ok(()) => {
            println!("Successfully imported database");
        }
        Err(e) => {
            eprintln!("An error occurred during database overwriting: {}", e);
        }
    }
    pw.zeroize();
}

pub fn add(matches: &ArgMatches) {   
    let mut secret = prompt_for_passwords("Insert the secret: ", 0, false);
    // Safe to unwrap due to default values
    let issuer = matches.value_of("issuer").unwrap();
    let label = matches.value_of("label").unwrap();
    let algorithm = matches.value_of("algorithm").unwrap();
    let digits: u64 = matches.value_of_t("digits").unwrap_or(6);
    let counter: u64 = matches.value_of_t("counter").unwrap_or_default();
    let hotp_type = matches.is_present("hotp");
    match database_management::add_element(secret.as_str(), issuer, label, algorithm, digits,counter,hotp_type) {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("An error occurred: {}", e)
    }
    secret.zeroize();
}

pub fn remove(matches: &ArgMatches) {
    let index: usize = matches.value_of_t_or_exit("index");
    match database_management::remove_element_from_db(index) {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("An error has occurred: {}", e)
    }
}

pub fn edit(matches: &ArgMatches) {
    let index: usize = matches.value_of_t_or_exit("index");
    let issuer = matches.value_of("issuer").unwrap_or("");
    let label = matches.value_of("label").unwrap_or("");
    let algorithm = matches.value_of("algorithm").unwrap_or("");
    let digits: u64 = matches.value_of_t("digits").unwrap_or(0);
    let counter: u64 = matches.value_of_t("counter").unwrap_or(0);
    let mut secret = match matches.is_present("change-secret") {
        true => prompt_for_passwords("Insert the secret (type ENTER to skip): ", 0, false),
        false => String::from(""),
    };
    match database_management::edit_element(index, &secret, &issuer, &label, &algorithm, digits, counter) {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("An error occurred: {}", e)
    }
    secret.zeroize();
}

pub fn export() {
    let export_result = database_management::export_database();
    match export_result {
        Ok(export_result) => {
            println!("Database was successfully exported at {}", export_result.to_str().unwrap_or("**Invalid path**"));
        }
        Err(e) => {
            eprintln!("An error occurred while exporting database: {}", e);
        }
    }
}

pub fn info(matches: &ArgMatches) {
    let index = matches.value_of_t_or_exit("index");
    match otp_helper::print_element_info(index) {
        Ok(()) => {}
        Err(e) => eprintln!("An error occurred: {}", e),
    }
}

pub fn change_password() {
    let mut old_password = cryptography::prompt_for_passwords("Old password: ", 8, false);
    let decrypted_text = database_management::read_decrypted_text(&old_password);
    old_password.zeroize();
    match decrypted_text {
        Ok(mut s) => {
            let mut new_password = cryptography::prompt_for_passwords("New password: ", 8, true);
            match database_management::overwrite_database_json(&s, &new_password) {
                Ok(()) => println!("Password changed"),
                Err(e) => eprintln!("An error has occurred: {}", e),
            }
            s.zeroize();
            new_password.zeroize();
        }
        Err(e) => {
            eprintln!("An error has occurred: {}", e);
        }
    }
} 