use std::path::PathBuf;

use crate::otp::otp_algorithm::OTPAlgorithm;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::otp::otp_type::OTPType;
use crate::{importers, utils};
use clap::ArgMatches;
use zeroize::Zeroize;

pub fn import(matches: &ArgMatches, database: &mut OTPDatabase) -> Result<String, String> {
    let path = matches.get_one::<String>("path").unwrap();

    let result = if matches.get_flag("cotp") || matches.get_flag("andotp") {
        importers::and_otp::import(path)
    } else if matches.get_flag("aegis") {
        importers::aegis::import(path)
    } else if matches.get_flag("aegis-encrypted") {
        let mut password =
            utils::prompt_for_passwords("Insert password for DB decryption: ", 0, false);
        let result = importers::aegis_encrypted::import(path, password.as_str());
        password.zeroize();
        result
    } else if matches.get_flag("freeotp-plus") {
        importers::freeotp_plus::import(path)
    } else if matches.get_flag("authy-exported") {
        importers::authy_remote_debug::import(path)
    } else if matches.get_flag("google-authenticator")
        || matches.get_flag("authy")
        || matches.get_flag("microsoft-authenticator")
        || matches.get_flag("freeotp")
    {
        importers::converted::import(path)
    } else {
        return Err(String::from("Invalid arguments provided"));
    };

    let elements = match result {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("An error occurred: {}", e));
        }
    };

    database.add_all(elements);
    Ok(String::from("Successfully imported database"))
}

pub fn add(matches: &ArgMatches, database: &mut OTPDatabase) -> Result<String, String> {
    let secret = utils::prompt_for_passwords("Insert the secret: ", 0, false);
    let type_ = OTPType::from(
        matches
            .get_one::<String>("type")
            .unwrap()
            .to_uppercase()
            .as_str(),
    );

    let otp_element = OTPElement {
        secret,
        issuer: matches.get_one::<String>("issuer").unwrap().clone(),
        label: matches.get_one::<String>("label").unwrap().clone(),
        digits: *matches.get_one("digits").unwrap_or(&6),
        type_,
        algorithm: OTPAlgorithm::from(
            matches
                .get_one::<String>("algorithm")
                .unwrap()
                .to_uppercase()
                .as_str(),
        ),
        period: *matches.get_one("period").unwrap_or(&6),
        counter: matches.get_one("counter").copied(),
        pin: matches.get_one::<String>("pin").map(|v| v.to_owned()),
    };

    if !otp_element.valid_secret() {
        return Err(String::from("Invalid secret."));
    }

    database.add_element(otp_element);
    Ok(String::from("Success."))
}

pub fn edit(matches: &ArgMatches, database: &mut OTPDatabase) -> Result<String, String> {
    let mut secret = match matches.get_flag("change-secret") {
        true => Some(utils::prompt_for_passwords("Insert the secret: ", 0, false)),
        false => None,
    };

    let index = *matches.get_one::<usize>("index").unwrap();
    let otp_element: Option<&OTPElement> = database.get_element(index);

    let issuer = matches.get_one::<String>("issuer").cloned();
    let label = matches.get_one::<String>("label").cloned();
    let digits = matches.get_one::<u64>("usize").copied();
    let period = matches.get_one::<u64>("period").copied();
    let counter = matches.get_one::<u64>("counter").copied();
    let pin = matches.get_one::<String>("label").cloned();

    match otp_element {
        Some(v) => {
            let mut element = v.clone();

            if let Some(v) = issuer {
                element.issuer = v;
            }
            if let Some(v) = label {
                element.label = v;
            }
            if let Some(v) = digits {
                element.digits = v;
            }
            if let Some(v) = period {
                element.period = v;
            }
            if counter.is_some() {
                element.counter = counter;
            }
            if pin.is_some() {
                element.pin = pin;
            }
            database.edit_element(index, element);
        }
        None => return Err(format!("No element found at index {}", index)),
    }

    secret.zeroize();
    Ok(String::from("Success."))
}

pub fn export(matches: &ArgMatches, database: &mut OTPDatabase) -> Result<String, String> {
    match database.export(PathBuf::from(matches.get_one::<&str>("path").unwrap())) {
        Ok(export_result) => Ok(format!(
            "Database was successfully exported at {}",
            export_result.to_str().unwrap_or("**Invalid path**")
        )),
        Err(e) => Err(format!("An error occurred while exporting database: {}", e)),
    }
}

pub fn change_password(database: &mut OTPDatabase) -> Result<String, String> {
    let mut new_password = utils::prompt_for_passwords("New password: ", 8, true);
    let r = match database.save_with_pw(&new_password) {
        Ok(()) => Ok(String::from("Password changed")),
        Err(e) => Err(format!("An error has occurred: {}", e)),
    };
    new_password.zeroize();
    r
}
