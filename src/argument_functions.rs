use std::path::PathBuf;

use crate::database_management::{self, check_secret};
use crate::otp::otp_element::{
    OTPAlgorithm, OTPDatabase, OTPElement, OTPType, CURRENT_DATABASE_VERSION,
};
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
    match database_management::overwrite_database(
        &OTPDatabase::new(CURRENT_DATABASE_VERSION, elements),
        &pw,
    ) {
        Ok(()) => {
            println!("Successfully imported database");
        }
        Err(e) => {
            eprintln!("An error occurred during database overwriting: {}", e);
        }
    }
    pw.zeroize();
}

pub fn add(matches: &ArgMatches, database: &mut OTPDatabase) {
    let secret = utils::prompt_for_passwords("Insert the secret: ", 0, false);
    let type_ = OTPType::from(matches.value_of("type").unwrap().to_uppercase().as_str());
    if check_secret(&secret, type_).is_err() {
        eprintln!("Invalid secret.");
        return;
    }

    let otp_element = OTPElement {
        secret,
        issuer: matches.get_one::<String>("issuer").unwrap().clone(),
        label: matches.get_one::<String>("label").unwrap().clone(),
        digits: *matches.get_one::<usize>("digits").unwrap_or(&6) as u64,
        type_,
        algorithm: match matches
            .value_of("algorithm")
            .unwrap()
            .to_uppercase()
            .as_str()
        {
            "SHA256" => OTPAlgorithm::OTPSha256,
            "SHA512" => OTPAlgorithm::OTPSha512,
            "MD5" => OTPAlgorithm::OTPMd5,
            _ => OTPAlgorithm::OTPSha1,
        },
        period: *matches.get_one::<usize>("period").unwrap_or(&6) as u64,
        counter: matches.get_one::<u64>("counter").map(|e| *e),
        pin: matches.get_one::<String>("pin").map(|v| v.to_owned()),
    };

    database.add_element(otp_element);
    println!("Success");
}

pub fn edit(matches: &ArgMatches, database: &mut OTPDatabase) {
    let mut secret = match matches.is_present("change-secret") {
        true => Some(utils::prompt_for_passwords("Insert the secret: ", 0, false)),
        false => None,
    };

    let index = *matches.get_one::<usize>("index").unwrap();
    let otp_element: Option<&OTPElement> = database.get_element(index);

    let issuer = matches.get_one::<String>("issuer").map(|e| e.clone());
    let label = matches.get_one::<String>("label").map(|e| e.clone());
    let digits = matches.get_one::<u64>("usize").map(|e| *e);
    let period = matches.get_one::<u64>("period").map(|e| *e);
    let counter = matches.get_one::<u64>("counter").map(|e| *e);
    let pin = matches.get_one::<String>("label").map(|e| e.clone());

    match otp_element {
        Some(v) => {
            let mut element = v.clone();

            if issuer.is_some() {
                element.issuer = issuer.unwrap();
            }
            if label.is_some() {
                element.label = label.unwrap();
            }
            if digits.is_some() {
                element.digits = digits.unwrap();
            }
            if period.is_some() {
                element.period = period.unwrap();
            }
            if counter.is_some() {
                element.counter = counter;
            }
            if pin.is_some() {
                element.pin = pin;
            }
            database.edit_element(index, element);
            println!("Success");
        }
        None => eprintln!("No element found at index {}", index),
    }

    secret.zeroize();
}

pub fn export(matches: &ArgMatches) {
    match database_management::export_database(PathBuf::from(
        matches.get_one::<&str>("path").unwrap(),
    )) {
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
