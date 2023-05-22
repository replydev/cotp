use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::{argument_functions, otp::otp_element::OTPDatabase};

pub fn args_parser(
    matches: ArgMatches,
    database: &mut OTPDatabase,
) -> Option<Result<String, String>> {
    match matches.subcommand() {
        Some(("add", add_matches)) => Some(argument_functions::add(add_matches, database)),
        Some(("edit", edit_matches)) => Some(argument_functions::edit(edit_matches, database)),
        Some(("import", import_matches)) => {
            Some(argument_functions::import(import_matches, database))
        }
        Some(("export", export_matches)) => {
            Some(argument_functions::export(export_matches, database))
        }
        Some(("passwd", _)) => Some(argument_functions::change_password(database)),
        Some((_, _)) => Some(Err(String::from("Invalid args"))),
        None => None,
    }
}

pub fn get_matches() -> ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(
            env!("CARGO_PKG_AUTHORS")
                .split(',')
                .next()
                .unwrap_or("replydev <commoncargo@tutanota.com>"),
        )
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("add")
                .about("Add a new OTP Code")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("otp_uri")
                        .short('u')
                        .long("otpuri")
                        .help("Add OTP code via an OTP URI")
                        .action(ArgAction::SetTrue)
                        .required_unless_present("issuer")
                )
                .arg(
                    Arg::new("type")
                        .short('t')
                        .long("type")
                        .help("Specify the OTP code type")
                        .num_args(1)
                        .value_parser(["TOTP", "HOTP", "STEAM", "YANDEX", "MOTP"])
                        .default_value("TOTP"),
                )
                .arg(
                    Arg::new("issuer")
                        .short('i')
                        .long("issuer")
                        .help("OTP Code issuer")
                        .num_args(1)
                        .required_unless_present("otp_uri"),
                )
                .arg(
                    Arg::new("label")
                        .short('l')
                        .long("label")
                        .help("OTP Code label")
                        .num_args(1)
                        .required(false)
                        .default_value(""),
                )
                .arg(
                    Arg::new("algorithm")
                        .short('a')
                        .long("algorithm")
                        .help("OTP Code algorithm")
                        .num_args(1)
                        .required(false)
                        .value_parser(["SHA1", "SHA256", "SHA512"])
                        .default_value("SHA1"),
                )
                .arg(
                    Arg::new("digits")
                        .short('d')
                        .long("digits")
                        .help("OTP Code digits")
                        .num_args(1)
                        .required(false)
                        .value_parser(value_parser!(u64))
                        .default_value_if("type", "STEAM", "5")
                        .default_value("6"),
                )
                .arg(
                    Arg::new("period")
                        .short('e')
                        .long("period")
                        .help("OTP Code period")
                        .num_args(1)
                        .required(false)
                        .value_parser(value_parser!(u64))
                        .default_value("30"),
                )
                .arg(
                    Arg::new("counter")
                        .short('c')
                        .long("counter")
                        .help("HOTP code counter")
                        .required_if_eq("type", "HOTP")
                        .num_args(1)
                        .value_parser(value_parser!(u64)),
                ).arg(
                Arg::new("pin")
                    .short('p')
                    .long("pin")
                    .help("Code pin (for Yandex and MOTP)")
                    .required_if_eq("type", "YANDEX")
                    .required_if_eq("type", "MOTP")
                    .num_args(1),
            ),
        )
        .subcommand(
            Command::new("edit")
                .about("Edit an OTP code")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("OTP Code index")
                        .num_args(1)
                        .value_parser(value_parser!(usize))
                        .required(true),
                )
                .arg(
                    Arg::new("issuer")
                        .short('s')
                        .long("issuer")
                        .help("OTP Code issuer")
                        .num_args(1)
                        .required_unless_present_any(["label", "algorithm", "digits", "counter"]),
                )
                .arg(
                    Arg::new("label")
                        .short('l')
                        .long("label")
                        .help("OTP Code label")
                        .num_args(1)
                        .required_unless_present_any(["issuer", "algorithm", "digits", "counter"]),
                )
                .arg(
                    Arg::new("algorithm")
                        .short('a')
                        .long("algorithm")
                        .help("OTP Code algorithm")
                        .num_args(1)
                        .required_unless_present_any(["label", "issuer", "digits", "counter"])
                        .value_parser(["SHA1", "SHA256", "SHA512"]),
                )
                .arg(
                    Arg::new("digits")
                        .short('d')
                        .long("digits")
                        .help("OTP Code digits")
                        .num_args(1)
                        .value_parser(value_parser!(u64))
                        .required_unless_present_any(["label", "algorithm", "issuer", "counter"]),
                )
                .arg(
                    Arg::new("period")
                        .short('e')
                        .long("period")
                        .help("OTP Code period")
                        .num_args(1)
                        .value_parser(value_parser!(u64))
                        .required_unless_present_any(["label", "algorithm", "issuer", "counter"]),
                )
                .arg(
                    Arg::new("counter")
                        .short('c')
                        .long("counter")
                        .help("HOTP code counter (only for HOTP codes)")
                        .num_args(1)
                        .value_parser(value_parser!(u64))
                        .required_unless_present_any(["label", "algorithm", "issuer", "digits"]),
                )
                .arg(
                    Arg::new("pin")
                        .short('p')
                        .long("pin")
                        .help("Code pin (for Yandex and MOTP)")
                        .num_args(1)
                )
                .arg(
                    Arg::new("change-secret")
                        .short('k')
                        .long("change-secret")
                        .help("Change the OTP code secret")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove an OTP code")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("OTP code index")
                        .num_args(1..)
                        .value_parser(value_parser!(u64))
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("import")
                .about("Import from backups")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("cotp")
                        .short('c')
                        .long("cotp")
                        .help("Import from cotp exported database")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("andotp")
                        .short('e')
                        .long("andotp")
                        .help("Import from andOTP backup")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "cotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "cotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("aegis")
                        .short('a')
                        .long("aegis")
                        .help("Import from Aegis backup")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "cotp",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "cotp",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("aegis-encrypted")
                        .short('k')
                        .long("aegis-encrypted")
                        .help("Import from Aegis encrypted backup")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "cotp",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "cotp",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis",
                        ]),
                )
                .arg(
                    Arg::new("freeotp-plus")
                        .short('f')
                        .long("freeotp-plus")
                        .help("Import from FreeOTP+ backup")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "cotp",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "cotp",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("freeotp")
                        .short('r')
                        .long("freeotp")
                        .help("Import from FreeOTP converted database")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "cotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "cotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("google-authenticator")
                        .short('g')
                        .long("google-authenticator")
                        .help("Import from Google Authenticator converted database")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "cotp",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "cotp",
                            "authy",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("authy")
                        .short('t')
                        .long("authy")
                        .help("Import from Authy converted database")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "cotp",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "cotp",
                            "authy-exported",
                            "microsoft-authenticator",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("microsoft-authenticator")
                        .short('m')
                        .long("microsoft-authenticator")
                        .help("Import from Microsoft Authenticator converted database")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "cotp",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "authy-exported",
                            "cotp",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("authy-exported")
                        .short('u')
                        .long("authy-exported")
                        .help("Import from Authy Database exported following https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93")
                        .action(ArgAction::SetTrue)
                        .required_unless_present_any([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "microsoft-authenticator",
                            "cotp",
                            "aegis-encrypted",
                        ])
                        .conflicts_with_all([
                            "andotp",
                            "aegis",
                            "freeotp-plus",
                            "freeotp",
                            "google-authenticator",
                            "authy",
                            "microsoft-authenticator",
                            "cotp",
                            "aegis-encrypted",
                        ]),
                )
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .help("Backup path")
                        .num_args(1)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("export").about("Export your database").arg(
                Arg::new("path")
                    .short('p')
                    .long("path")
                    .help("Export file path")
                    .num_args(1)
                    .required(false)
                    .default_value("."),
            ),
        )
        .subcommand(Command::new("passwd").about("Change your database password"))
        .get_matches()
}
