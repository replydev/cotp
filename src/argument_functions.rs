use std::path::PathBuf;

use crate::database_management;
use crate::{importers, utils};
use clap::ArgMatches;
use zeroize::Zeroize;

pub fn import(matches: &ArgMatches) {
    let path = matches.value_of("path").unwrap();

    let result = if matches.is_present("cotp") || matches.is_present("andotp") {
        importers::and_otp::import(path)
    } else if matches.is_present("aegis") {
        importers::aegis::import(path)
    } else if matches.is_present("aegis-encrypted") {
        let mut password =
            utils::prompt_for_passwords("Insert password for DB decryption: ", 0, false);
        let result = importers::aegis_encrypted::import(path, password.as_str());
        password.zeroize();
        result
    } else if matches.is_present("freeotp-plus") {
        importers::freeotp_plus::import(path)
    } else if matches.is_present("authy-exported") {
        importers::authy_remote_debug::import(path)
    } else if matches.is_present("google-authenticator")
        || matches.is_present("authy")
        || matches.is_present("microsoft-authenticator")
        || matches.is_present("freeotp")
    {
        importers::converted::import(path)
    } else {
        eprintln!("Invalid arguments provided");
        return;
    };

    let elements = match result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            return;
        }
    };

    let mut pw = utils::prompt_for_passwords("Choose a password: ", 8, true);
    match database_management::overwrite_database(&elements, &pw) {
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
    let mut secret = utils::prompt_for_passwords("Insert the secret: ", 0, false);
    match database_management::add_element(
        secret.as_str(),
        // Safe to unwrap due to default values
        matches.value_of("issuer").unwrap(),
        matches.value_of("label").unwrap(),
        matches.value_of("algorithm").unwrap(),
        matches.value_of_t("digits").unwrap_or(6),
        matches.value_of_t("counter").unwrap_or_default(),
        matches.value_of("type").unwrap(),
    ) {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("An error occurred: {}", e),
    }
    secret.zeroize();
}

pub fn remove(matches: &ArgMatches) {
    match database_management::remove_element_from_db(
        matches
            .values_of("index")
            .unwrap()
            .map(|s| s.parse::<usize>().unwrap())
            .collect(),
    ) {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("An error has occurred: {}", e),
    }
}

pub fn edit(matches: &ArgMatches) {
    let mut secret = match matches.is_present("change-secret") {
        true => utils::prompt_for_passwords("Insert the secret (type ENTER to skip): ", 0, false),
        false => String::from(""),
    };
    match database_management::edit_element(
        matches.value_of_t_or_exit("index"),
        secret.as_str(),
        matches.value_of("issuer").unwrap_or(""),
        matches.value_of("label").unwrap_or(""),
        matches.value_of("algorithm").unwrap_or(""),
        matches.value_of_t("digits").unwrap_or(0),
        matches.value_of_t("counter").unwrap_or(0),
    ) {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("An error occurred: {}", e),
    }
    secret.zeroize();
}

pub fn export(matches: &ArgMatches) {
    match database_management::export_database(PathBuf::from(matches.value_of("path").unwrap())) {
        Ok(export_result) => {
            println!(
                "Database was successfully exported at {}",
                export_result.to_str().unwrap_or("**Invalid path**")
            );
        }
        Err(e) => {
            eprintln!("An error occurred while exporting database: {}", e);
        }
    }
}

pub fn info(matches: &ArgMatches) {
    if let Err(e) = database_management::print_element_info(matches.value_of_t_or_exit("index")) {
        eprintln!("An error occurred: {}", e);
    }
}

pub fn search(matches: &ArgMatches) {
    match database_management::print_elements_matching(
        matches.value_of("issuer"),
        matches.value_of("label"),
    ) {
        Ok(()) => {}
        Err(e) => eprintln!("An error occurred: {}", e),
    }
}

pub fn change_password() {
    let mut old_password = utils::prompt_for_passwords("Old password: ", 8, false);
    let result = database_management::read_decrypted_text(&old_password);
    old_password.zeroize();
    match result {
        Ok((mut s, mut key, _salt)) => {
            key.zeroize();
            let mut new_password = utils::prompt_for_passwords("New password: ", 8, true);
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

pub fn qrcode(matches: &ArgMatches) {
    let issuer: String = matches.value_of_t_or_exit("issuer");
    if let Err(e) = database_management::show_qr_code(issuer) {
        eprintln!("An error has occurred: {}", e);
    }
}
