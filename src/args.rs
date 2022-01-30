use clap::{App, AppSettings, Arg, ArgMatches};

use crate::argument_functions;

pub fn args_parser() -> bool {
    match get_matches().subcommand() {
        Some(("add",add_matches)) => argument_functions::add(add_matches),
        Some(("edit",edit_matches)) => argument_functions::edit(edit_matches),
        Some(("remove",remove_matches)) => argument_functions::remove(remove_matches),
        Some(("import",import_matches)) => argument_functions::import(import_matches),
        Some(("info",info_matches)) => argument_functions::info(info_matches),
        Some(("export",export_matches)) => argument_functions::export(export_matches),
        Some(("qrcode",qrcode_matches)) => argument_functions::qrcode(qrcode_matches),
        Some(("passwd",_)) => argument_functions::change_password(),
        Some(("search",search_matches)) => argument_functions::search(search_matches),
        _ => return true,
    }
    false
}

fn get_matches() -> ArgMatches{
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS").split(',').next().unwrap_or("replydev <commoncargo@tutanota.com>"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        //.license("GPL3")
        .subcommand(
            App::new("add")
                .about("Add a new OTP Code")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::new("hotp")
                    .short('o')
                    .long("hotp")
                    .help("Specify this is an HOTP Code")
                    .takes_value(false)
                    .requires("counter")
                )
                .arg(
                    Arg::new("issuer")
                    .short('i')
                    .long("issuer")
                    .help("OTP Code issuer")
                    .takes_value(true)
                    .required(true)
                )
                .arg(
                    Arg::new("label")
                    .short('l')
                    .long("label")
                    .help("OTP Code label")
                    .takes_value(true)
                    .required(false)
                    .default_value("")
                )
                .arg(
                    Arg::new("algorithm")
                    .short('a')
                    .long("algoritmh")
                    .help("OTP Code algorithm")
                    .takes_value(true)
                    .required(false)
                    .possible_values(&["SHA1","SHA256","SHA512"])
                    .default_value("SHA1")
                )
                .arg(
                    Arg::new("digits")
                    .short('d')
                    .long("digits")
                    .help("OTP Code digits")
                    .takes_value(true)
                    .required(false)
                    .default_value("6")
                )
                .arg(
                    Arg::new("counter")
                    .short('c')
                    .long("counter")
                    .help("HOTP code counter")
                    .takes_value(true)
                )
        )
        .subcommand(
            App::new("edit")
                .about("Edit an OTP code")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::new("index")
                    .short('i')
                    .long("index")
                    .help("OTP Code index")
                    .takes_value(true)
                    .required(true)
                )
                .arg(
                    Arg::new("issuer")
                    .short('s')
                    .long("issuer")
                    .help("OTP Code issuer")
                    .takes_value(true)
                    .required_unless_present_any(["label","algorithm","digits","counter"])
                )
                .arg(
                    Arg::new("label")
                    .short('l')
                    .long("label")
                    .help("OTP Code label")
                    .takes_value(true)
                    .required_unless_present_any(["issuer","algorithm","digits","counter"])
                )
                .arg(
                    Arg::new("algorithm")
                    .short('a')
                    .long("algoritmh")
                    .help("OTP Code algorithm")
                    .takes_value(true)
                    .required_unless_present_any(["label","issuer","digits","counter"])
                    .possible_values(&["SHA1","SHA256","SHA512"])
                )
                .arg(
                    Arg::new("digits")
                    .short('d')
                    .long("digits")
                    .help("OTP Code digits")
                    .takes_value(true)
                    .required_unless_present_any(["label","algorithm","issuer","counter"])
                )
                .arg(
                    Arg::new("counter")
                    .short('c')
                    .long("counter")
                    .help("HOTP code counter (only for HOTP codes)")
                    .takes_value(true)
                    .required_unless_present_any(["label","algorithm","issuer","digits"])
                )
                .arg(
                    Arg::new("change-secret")
                    .short('k')
                    .long("change-secret")
                    .help("Change the OTP code secret")
                    .takes_value(false)
                )
        )
        .subcommand(
            App::new("remove")
                .about("Remove an OTP code")
                .setting(AppSettings::ArgRequiredElseHelp) 
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("OTP code index")
                        .takes_value(true)
                        .required(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            App::new("import")
                .about("Import from backups")
                .setting(AppSettings::ArgRequiredElseHelp) 
                .arg(
                    Arg::new("cotp")
                        .short('c')
                        .long("cotp")
                        .help("Import from cotp exported database")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","aegis","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","aegis","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("andotp")
                        .short('e')
                        .long("andotp")
                        .help("Import from andOTP backup")
                        .takes_value(false)
                        .required_unless_present_any(&["cotp","aegis","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["cotp","aegis","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("aegis")
                        .short('a')
                        .long("aegis")
                        .help("Import from Aegis backup")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","cotp","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","cotp","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("aegis-encrypted")
                        .short('k')
                        .long("aegis-encrypted")
                        .help("Import from Aegis encrypted backup")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","cotp","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis"])
                        .conflicts_with_all(&["andotp","cotp","freeotp-plus","freeotp","google-authenticator","authy","microsoft-authenticator","aegis"])
                )
                .arg(
                    Arg::new("freeotp-plus")
                        .short('f')
                        .long("freeotp-plus")
                        .help("Import from FreeOTP+ backup")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","aegis","cotp","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","aegis","cotp","freeotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("freeotp")
                        .short('r')
                        .long("freeotp")
                        .help("Import from FreeOTP converted database")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","aegis","freeotp-plus","cotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","aegis","freeotp-plus","cotp","google-authenticator","authy","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("google-authenticator")
                        .short('g')
                        .long("google-authenticator")
                        .help("Import from Google Authenticator converted database")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","aegis","freeotp-plus","freeotp","cotp","authy","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","aegis","freeotp-plus","freeotp","cotp","authy","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("authy")
                        .short('t')
                        .long("authy")
                        .help("Import from Authy converted database")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","aegis","freeotp-plus","freeotp","google-authenticator","cotp","microsoft-authenticator","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","aegis","freeotp-plus","freeotp","google-authenticator","cotp","microsoft-authenticator","aegis-encrypted"])
                )
                .arg(
                    Arg::new("microsoft-authenticator")
                        .short('m')
                        .long("microsoft-authenticator")
                        .help("Import from Microsoft Authenticator converted database")
                        .takes_value(false)
                        .required_unless_present_any(&["andotp","aegis","freeotp-plus","freeotp","google-authenticator","authy","cotp","aegis-encrypted"])
                        .conflicts_with_all(&["andotp","aegis","freeotp-plus","freeotp","google-authenticator","authy","cotp","aegis-encrypted"])
                )
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .help("Backup path")
                        .takes_value(true)
                        .required(true),
                )
                
        )
        .subcommand(
            App::new("export")
                .about("Export your database")
                .arg(
                    Arg::new("path")
                    .short('p')
                    .long("path")
                    .help("Export file path")
                    .takes_value(true)
                    .required(false)
                    .default_value(".")
                )
        )
        .subcommand(
            App::new("info")
                .about("Show OTP code information")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("OTP code index")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("search")
                .about("Show OTP code for matching database entries")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::new("issuer")
                        .short('i')
                        .long("issuer")
                        .help("Search database by issuer")
                        .takes_value(true)
                        .required_unless_present("label"),
                )
                .arg(
                    Arg::new("label")
                        .short('l')
                        .long("label")
                        .help("Search database by label")
                        .takes_value(true)
                        .required_unless_present("issuer"),
                ),
        )
        .subcommand(
            App::new("passwd")
            .about("Change your database password")
        )
        .subcommand(
            App::new("qrcode")
                .setting(AppSettings::ArgRequiredElseHelp)
                .about("Show otpauth QRCode")
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("OTP Code index")
                        .takes_value(true)
                        .required(true)
                )
        )
        .get_matches()
}


