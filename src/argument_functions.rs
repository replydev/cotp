use crate::args::{AddArgs, EditArgs, ExportArgs, ImportArgs};
use crate::otp::otp_element::{FromOtpUri, OTPDatabase, OTPElement};
use crate::{importers, utils};
use zeroize::Zeroize;

pub fn import(matches: ImportArgs, database: &mut OTPDatabase) -> Result<String, String> {
    let path = matches.path;

    let backup_type = matches.backup_type;

    let result = if backup_type.cotp {
        importers::cotp::import(path)
    } else if backup_type.andotp {
        importers::and_otp::import(path)
    } else if backup_type.aegis {
        importers::aegis::import(path)
    } else if backup_type.aegis_encrypted {
        let mut password = utils::password("Insert your Aegis password: ", 0);
        let result = importers::aegis_encrypted::import(path, password.as_str());
        password.zeroize();
        result
    } else if backup_type.freeotp_plus {
        importers::freeotp_plus::import(path)
    } else if backup_type.authy_exported {
        importers::authy_remote_debug::import(path)
    } else if backup_type.google_authenticator
        || backup_type.authy
        || backup_type.microsoft_authenticator
        || backup_type.freeotp
    {
        importers::converted::import(path)
    } else {
        return Err(String::from("Invalid arguments provided"));
    };

    let elements = match result {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("An error occurred: {e}"));
        }
    };

    database.add_all(elements);
    Ok(String::from("Successfully imported database"))
}

pub fn add(matches: AddArgs, database: &mut OTPDatabase) -> Result<String, String> {
    let otp_element = if matches.otp_uri {
        let mut otp_uri = rpassword::prompt_password("Insert the otp uri: ").unwrap();
        let result = OTPElement::from_otp_uri(otp_uri.as_str());
        otp_uri.zeroize();
        result?
    } else {
        get_from_args(matches)?
    };
    if !otp_element.valid_secret() {
        return Err(String::from("Invalid secret."));
    }

    database.add_element(otp_element);
    Ok(String::from("Success."))
}

fn get_from_args(matches: AddArgs) -> Result<OTPElement, String> {
    let secret = rpassword::prompt_password("Insert the secret: ")
        .map_err(|e| format!("Error during password insertion: {:?}", e))?;
    Ok(map_args_to_code(secret, matches))
}

fn map_args_to_code(secret: String, matches: AddArgs) -> OTPElement {
    OTPElement {
        secret,
        issuer: matches.issuer.unwrap(),
        label: matches.label,
        digits: matches.digits,
        type_: matches.otp_type,
        algorithm: matches.algorithm,
        period: matches.period,
        counter: matches.counter,
        pin: matches.pin,
    }
}

pub fn edit(matches: EditArgs, database: &mut OTPDatabase) -> Result<String, String> {
    let secret = matches
        .change_secret
        .then(|| rpassword::prompt_password("Insert the secret: ").unwrap());

    // User provides row number from dashboard which is equal to the array index plus one
    let index = matches.index;

    if let Some(real_index) = index.checked_sub(1) {
        if real_index >= database.elements_ref().len() {
            return Err(format!("{index} is an invalid index"));
        }

        match database.mut_element(real_index) {
            Some(element) => {
                if let Some(v) = matches.issuer {
                    element.issuer = v;
                }
                if let Some(v) = matches.label {
                    element.label = v;
                }
                if let Some(v) = matches.digits {
                    element.digits = v;
                }
                if let Some(v) = matches.period {
                    element.period = v;
                }
                if let Some(v) = matches.algorithm {
                    element.algorithm = v;
                }
                if matches.counter.is_some() {
                    element.counter = matches.counter;
                }
                if matches.pin.is_some() {
                    element.pin = matches.pin;
                }
                if let Some(s) = secret {
                    element.secret = s;
                }
                database.mark_modified();
            }
            None => return Err(format!("No element found at index {index}")),
        }
        Ok(String::from("Success."))
    } else {
        Err(format! {"{index} is an invalid index"})
    }
}

pub fn export(matches: ExportArgs, database: &mut OTPDatabase) -> Result<String, String> {
    match database.export(matches.path) {
        Ok(export_result) => Ok(format!(
            "Database was successfully exported at {}",
            export_result.to_str().unwrap_or("**Invalid path**")
        )),
        Err(e) => Err(format!("An error occurred while exporting database: {e}")),
    }
}

pub fn change_password(database: &mut OTPDatabase) -> Result<String, String> {
    let mut new_password = utils::verified_password("New password: ", 8);
    let r = match database.save_with_pw(&new_password) {
        Ok(_) => Ok(String::from("Password changed")),
        Err(e) => Err(format!("An error has occurred: {e}")),
    };
    new_password.zeroize();
    r
}
