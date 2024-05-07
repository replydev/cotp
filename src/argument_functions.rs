use crate::args::{AddArgs, EditArgs, ExportArgs, ExtractArgs, ImportArgs, ListArgs};
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
use crate::{clipboard, utils};
use color_eyre::eyre::{eyre, ErrReport};
use zeroize::Zeroize;

pub fn import(matches: ImportArgs, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
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
        return Err(eyre!("Invalid arguments provided"));
    };

    let elements = result.map_err(|e| eyre!("{e}"))?;

    database.add_all(elements);
    Ok(database)
}

pub fn add(matches: AddArgs, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    let otp_element = if matches.otp_uri {
        let mut otp_uri = rpassword::prompt_password("Insert the otp uri: ").unwrap();
        let result = OTPElement::from_otp_uri(otp_uri.as_str());
        otp_uri.zeroize();
        result?
    } else {
        get_from_args(matches)?
    };
    if !otp_element.valid_secret() {
        return Err(ErrReport::msg("Invalid secret."));
    }

    database.add_element(otp_element);
    Ok(database)
}

fn get_from_args(matches: AddArgs) -> color_eyre::Result<OTPElement> {
    let secret = rpassword::prompt_password("Insert the secret: ").map_err(ErrReport::from)?;
    Ok(map_args_to_code(secret, matches))
}

fn map_args_to_code(secret: String, matches: AddArgs) -> OTPElement {
    OTPElement {
        secret,
        issuer: matches.issuer,
        label: matches.label.unwrap(),
        digits: matches.digits,
        type_: matches.otp_type,
        algorithm: matches.algorithm,
        period: matches.period,
        counter: matches.counter,
        pin: matches.pin,
    }
}

pub fn edit(matches: EditArgs, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    let secret = matches
        .change_secret
        .then(|| rpassword::prompt_password("Insert the secret: ").unwrap());

    // User provides row number from dashboard which is equal to the array index plus one
    let index = matches.index;

    if let Some(real_index) = index.checked_sub(1) {
        if real_index >= database.elements_ref().len() {
            return Err(eyre!("{index} is an invalid index"));
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
            None => return Err(eyre!("No element found at index {index}")),
        }
        Ok(database)
    } else {
        Err(eyre!("{index} is an invalid index"))
    }
}

pub fn list(matches: ListArgs, mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    todo!()
}

pub fn export(matches: ExportArgs, database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
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
    } else if export_format.otp_uri {
        let otp_uri_list: OtpUriList = (&database).into();
        do_export(&otp_uri_list, exported_path)
    } else if export_format.freeotp_plus {
        let freeotp_plus: FreeOTPPlusJson = (&database).try_into()?;
        do_export(&freeotp_plus, exported_path)
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
    .map_err(|e| eyre!("An error occurred while exporting database: {e}"))
}

pub fn extract(args: ExtractArgs, database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    let first_with_filters = database
        .elements
        .iter()
        .enumerate()
        .find(|(index, code)| filter_extract(&args, index, code))
        .map(|(_, code)| code);

    if let Some(otp) = first_with_filters {
        let code = otp.get_otp_code()?;
        println!("{}", code);
        if args.copy_to_clipboard {
            let _ = clipboard::copy_string_to_clipboard(code.as_str())?;
            println!("Copied to clipboard");
        }
        Ok(database)
    } else {
        Err(eyre!("No such code found with these fields"))
    }
}

pub fn change_password(mut database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
    let mut new_password = utils::verified_password("New password: ", 8);
    database
        .save_with_pw(&new_password)
        .map_err(ErrReport::from)?;
    new_password.zeroize();
    Ok(database)
}

fn filter_extract(args: &ExtractArgs, index: &usize, code: &OTPElement) -> bool {
    let match_by_index = args.index.map_or(true, |i| i == *index);

    let match_by_issuer = args.issuer.as_ref().map_or(true, |issuer| {
        code.issuer.to_lowercase() == issuer.to_lowercase()
    });

    let match_by_label = args.label.as_ref().map_or(true, |label| {
        code.label.to_lowercase() == label.to_lowercase()
    });

    match_by_index && match_by_issuer && match_by_label
}
