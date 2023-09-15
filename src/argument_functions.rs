use crate::args::{AddArgs, EditArgs, ExportArgs, ImportArgs};
use crate::exporters::do_export;
use crate::exporters::otp_uri::OtpUriList;
use crate::importers::aegis::AegisJson;
use crate::importers::aegis_encrypted::AegisEncryptedDatabase;
use crate::importers::authy_remote_debug::AuthyExportedList;
use crate::importers::converted::ConvertedJsonList;
use crate::importers::freeotp_plus::FreeOTPPlusJson;
use crate::importers::importer::import_from_path;
use crate::otp::from_otp_uri::FromOtpUri;
use crate::otp::otp_element::{OTPDatabase, OTPElement};
use crate::utils;
use zeroize::Zeroize;

pub fn import(matches: ImportArgs, mut database: OTPDatabase) -> Result<OTPDatabase, String> {
    let path = matches.path;

    let backup_type = matches.backup_type;

    let result = if backup_type.cotp {
        import_from_path::<OTPDatabase>(path)
    } else if backup_type.andotp {
        import_from_path::<Vec<OTPElement>>(path)
    } else if backup_type.aegis {
        import_from_path::<AegisJson>(path)
    } else if backup_type.aegis_encrypted {
        import_from_path::<AegisEncryptedDatabase>(path)
    } else if backup_type.freeotp_plus {
        import_from_path::<FreeOTPPlusJson>(path)
    } else if backup_type.authy_exported {
        import_from_path::<AuthyExportedList>(path)
    } else if backup_type.google_authenticator
        || backup_type.authy
        || backup_type.microsoft_authenticator
        || backup_type.freeotp
    {
        import_from_path::<ConvertedJsonList>(path)
    } else if backup_type.otp_uri {
        import_from_path::<OtpUriList>(path)
    } else {
        return Err(String::from("Invalid arguments provided"));
    };

    let elements = result.map_err(|e| format!("{e}"))?;

    database.add_all(elements);
    Ok(database)
}

pub fn add(matches: AddArgs, mut database: OTPDatabase) -> Result<OTPDatabase, String> {
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
    Ok(database)
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

pub fn edit(matches: EditArgs, mut database: OTPDatabase) -> Result<OTPDatabase, String> {
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
        Ok(database)
    } else {
        Err(format! {"{index} is an invalid index"})
    }
}

pub fn export(matches: ExportArgs, database: OTPDatabase) -> Result<OTPDatabase, String> {
    let export_format = matches.format.unwrap_or_default();
    let exported_path = if matches.path.is_dir() {
        matches.path.join("exported.cotp")
    } else {
        matches.path
    };

    if export_format.cotp {
        do_export(&database, exported_path)
    } else if export_format.andotp {
        let andotp: &Vec<OTPElement> = (&database).into();
        do_export(&andotp, exported_path)
    } else if export_format.otpuri {
        let otp_uri_list: OtpUriList = (&database).into();
        do_export(&otp_uri_list, exported_path)
    } else {
        unreachable!("Unreachable code");
    }
    .map(|path| {
        println!(
            "Exported to path: {}",
            path.to_str().unwrap_or("Failed to encode path")
        );
        database
    })
    .map_err(|e| format!("An error occurred while exporting database: {e}"))
}

pub fn change_password(mut database: OTPDatabase) -> Result<OTPDatabase, String> {
    let mut new_password = utils::verified_password("New password: ", 8);
    database
        .save_with_pw(&new_password)
        .map_err(|e| format!("An error has occurred: {e}"))?;
    new_password.zeroize();
    Ok(database)
}
