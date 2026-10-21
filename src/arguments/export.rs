use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::eyre;

use crate::{
    exporters::{do_export, otp_uri::OtpUriList},
    importers::freeotp_plus::FreeOTPPlusJson,
    otp::otp_element::{OTPDatabase, OTPElement},
};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ExportArgs {
    /// Acknowledge that exported files contain unencrypted authentication secrets
    #[arg(
        long,
        required_unless_present = "aegis_encrypted",
        conflicts_with = "aegis_encrypted"
    )]
    pub plaintext: bool,

    /// Export only these account IDs (repeatable)
    #[arg(long = "id")]
    pub ids: Vec<String>,
    /// Export only accounts containing this tag
    #[arg(long)]
    pub tag: Option<String>,
    /// Match issuer, label, alias, or tags (case insensitive substring)
    #[arg(long)]
    pub query: Option<String>,
    /// Read the Aegis export password from a private file
    #[arg(long, requires = "aegis_encrypted")]
    pub export_password_file: Option<PathBuf>,

    /// Export file path
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Export format
    #[command(flatten)]
    pub format: Option<ExportFormat>,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
pub struct ExportFormat {
    /// Export an unencrypted Aegis backup
    #[arg(long)]
    pub aegis: bool,
    /// Export an encrypted Aegis backup with a separate password
    #[arg(long = "aegis-encrypted")]
    pub aegis_encrypted: bool,

    /// Export into cotp backup
    #[arg(short, long)]
    pub cotp: bool,

    /// Export into andOTP backup
    #[arg(short = 'e', long)]
    pub andotp: bool,

    /// Export into an OTP URI
    #[arg(short, long = "otp-uri")]
    pub otp_uri: bool,

    /// Export into the `FreeOTP`+ database format
    #[arg(short, long = "freeotp-plus")]
    pub freeotp_plus: bool,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self {
            aegis: false,
            aegis_encrypted: false,
            cotp: true,
            andotp: false,
            otp_uri: false,
            freeotp_plus: false,
        }
    }
}

impl SubcommandExecutor for ExportArgs {
    fn run_command(self, database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let export_format = self.format.unwrap_or_default();
        for id in &self.ids {
            if !database.elements_ref().iter().any(|e| &e.id == id) {
                return Err(eyre!("Export account ID does not exist"));
            }
        }
        let elements: Vec<OTPElement> = database
            .elements_ref()
            .iter()
            .filter(|e| {
                (self.ids.is_empty() || self.ids.contains(&e.id))
                    && self
                        .tag
                        .as_ref()
                        .is_none_or(|tag| e.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
                    && self.query.as_ref().is_none_or(|query| {
                        let query = query.to_lowercase();
                        [&e.issuer, &e.label]
                            .into_iter()
                            .chain(e.alias.iter())
                            .chain(e.tags.iter())
                            .any(|value| value.to_lowercase().contains(&query))
                    })
            })
            .cloned()
            .collect();
        if elements.is_empty() {
            return Err(eyre!("No accounts match the export selection"));
        }
        let selected = OTPDatabase::from(elements);
        let exported_path = if self.path.is_dir() {
            self.path.join(if export_format.aegis_encrypted {
                "exported.aegis.json"
            } else {
                "exported.plaintext.json"
            })
        } else {
            self.path
        };

        if export_format.aegis || export_format.aegis_encrypted {
            let password = if export_format.aegis_encrypted {
                Some(zeroize::Zeroizing::new(
                    if let Some(path) = self.export_password_file {
                        use std::io::Read;
                        let mut password = zeroize::Zeroizing::new(String::new());
                        std::fs::File::open(path)?
                            .take(4097)
                            .read_to_string(&mut password)?;
                        if password.len() > 4096 {
                            return Err(eyre!("Export password file is too large"));
                        }
                        password.trim_end_matches(['\r', '\n']).to_owned()
                    } else {
                        crate::utils::verified_password("New Aegis export password: ", 8)?
                    },
                ))
            } else {
                None
            };
            let value =
                crate::exporters::aegis::encode(&selected, password.as_ref().map(|p| p.as_str()))?;
            do_export(&value, exported_path)
        } else if export_format.cotp {
            do_export(&selected, exported_path)
        } else if export_format.andotp {
            let andotp: &Vec<OTPElement> = (&selected).into();
            do_export(&andotp, exported_path)
        } else if export_format.otp_uri {
            let otp_uri_list: OtpUriList = (&selected).into();
            do_export(&otp_uri_list, exported_path)
        } else if export_format.freeotp_plus {
            let freeotp_plus: FreeOTPPlusJson = (&selected).try_into()?;
            do_export(&freeotp_plus, exported_path)
        } else {
            unreachable!("Unreachable code");
        }
        .map(|path| {
            eprintln!(
                "Exported to path: {}",
                path.to_str().unwrap_or("Failed to encode path")
            );
            database
        })
        .map_err(|e| eyre!("An error occurred while exporting database: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::from_otp_uri::FromOtpUri;

    #[test]
    fn selection_combines_ids_query_and_tag_and_preserves_metadata() {
        let mut database = OTPDatabase::default();
        for (issuer, tag) in [
            ("Example", "work"),
            ("Example", "personal"),
            ("Other", "work"),
        ] {
            let mut e =
                OTPElement::from_otp_uri(&format!("otpauth://totp/{issuer}:alice?secret=JBSWY3DP"))
                    .unwrap();
            e.tags.push(tag.to_owned());
            database.add_element(e).unwrap();
        }
        let first_id = database.elements_ref()[0].id.clone();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("selected.json");
        let returned = ExportArgs {
            plaintext: true,
            ids: database
                .elements_ref()
                .iter()
                .map(|e| e.id.clone())
                .collect(),
            tag: Some("WORK".into()),
            query: Some("EXAMPLE".into()),
            export_password_file: None,
            path: path.clone(),
            format: None,
        }
        .run_command(database)
        .unwrap();
        assert_eq!(returned.elements_ref().len(), 3);
        let exported: OTPDatabase = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(exported.elements_ref().len(), 1);
        assert_eq!(exported.elements_ref()[0].id, first_id);
        assert_eq!(exported.elements_ref()[0].tags, ["work"]);
        let missing = ExportArgs {
            plaintext: true,
            ids: vec!["missing".into()],
            tag: None,
            query: None,
            export_password_file: None,
            path: directory.path().join("missing.json"),
            format: None,
        }
        .run_command(returned);
        assert!(missing.is_err());
        assert!(!directory.path().join("missing.json").exists());
    }
}
