use crate::otp::otp_element::OTPDatabase;
use crate::{clipboard, otp::otp_element::OTPElement};
use clap::Args;
use color_eyre::eyre::eyre;
use globset::{Glob, GlobMatcher};

use super::SubcommandExecutor;

#[derive(Args)]
pub struct ExtractArgs {
    /// Code Index
    #[arg(short, long, required_unless_present_any = ["issuer", "label"])]
    pub index: Option<usize>,

    /// Code issuer
    #[arg(short = 's', long, required_unless_present_any = ["index", "label"])]
    pub issuer: Option<String>,

    /// Code label
    #[arg(short, long, required_unless_present_any = ["index", "issuer"])]
    pub label: Option<String>,

    /// Copy the code to the clipboard
    #[arg(short, long = "copy-clipboard", default_value_t = false)]
    pub copy_to_clipboard: bool,
}

// Contains glob filters for each field we can filter on
struct ExtractFilterGlob {
    issuer_glob: Option<GlobMatcher>,
    label_glob: Option<GlobMatcher>,
    index: Option<usize>,
}

impl TryFrom<ExtractArgs> for ExtractFilterGlob {
    type Error = color_eyre::eyre::ErrReport;

    fn try_from(value: ExtractArgs) -> Result<Self, Self::Error> {
        let issuer_glob = if value.issuer.is_some() {
            Some(Glob::new(&value.issuer.unwrap())?.compile_matcher())
        } else {
            None
        };

        let label_glob = if value.label.is_some() {
            Some(Glob::new(&value.label.unwrap())?.compile_matcher())
        } else {
            None
        };

        Ok(Self {
            issuer_glob,
            label_glob,
            index: value.index,
        })
    }
}

impl SubcommandExecutor for ExtractArgs {
    fn run_command(self, otp_database: OTPDatabase) -> color_eyre::Result<OTPDatabase> {
        let copy_to_clipboard = self.copy_to_clipboard;
        let globbed: ExtractFilterGlob = self.try_into()?;

        let first_with_filters = find_match(&otp_database, globbed);

        if let Some(otp) = first_with_filters {
            let code = otp.get_otp_code()?;
            println!("{code}");
            if copy_to_clipboard {
                let _ = clipboard::copy_string_to_clipboard(code.as_str())?;
                println!("Copied to clipboard");
            }
            Ok(otp_database)
        } else {
            Err(eyre!("No such code found with these fields"))
        }
    }
}

fn find_match(otp_database: &OTPDatabase, globbed: ExtractFilterGlob) -> Option<&OTPElement> {
    otp_database
        .elements
        .iter()
        .enumerate()
        .find(|(index, code)| filter_extract(&globbed, *index, code))
        .map(|(_, code)| code)
}

fn filter_extract(args: &ExtractFilterGlob, index: usize, candidate: &OTPElement) -> bool {
    let match_by_index = args.index.is_none_or(|i| i == index);

    let match_by_issuer = args
        .issuer_glob
        .as_ref()
        .is_none_or(|issuer| issuer.is_match(&candidate.issuer));

    let match_by_label = args
        .label_glob
        .as_ref()
        .is_none_or(|label| label.is_match(&candidate.label));

    match_by_index && match_by_issuer && match_by_label
}

#[cfg(test)]
mod tests {
    use globset::Glob;

    use crate::otp::otp_element::{OTPDatabase, OTPElementBuilder};

    use super::{find_match, ExtractFilterGlob};

    #[test]
    fn test_glob_filtering_good_issuer() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer")
                .label("test-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer2")
                .label("test-label2")
                .secret("AA")
                .build()
                .unwrap(),
        );

        let filter = ExtractFilterGlob {
            issuer_glob: Some(Glob::new("test-iss*").unwrap().compile_matcher()),
            label_glob: None,
            index: None,
        };

        // Act
        let found_match = find_match(&otp_database, filter);

        // Assert
        assert!(found_match.is_some());
    }

    #[test]
    fn test_glob_filtering_good_label() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer")
                .label("test-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer2")
                .label("test-label2")
                .secret("AA")
                .build()
                .unwrap(),
        );

        let filter = ExtractFilterGlob {
            issuer_glob: None,
            label_glob: Some(Glob::new("test-la*").unwrap().compile_matcher()),
            index: None,
        };

        // Act
        let found_match = find_match(&otp_database, filter);

        // Assert
        assert!(found_match.is_some());
    }

    #[test]
    fn test_glob_filtering_no_match() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer")
                .label("test-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer2")
                .label("test-label2")
                .secret("AA")
                .build()
                .unwrap(),
        );

        let filter = ExtractFilterGlob {
            issuer_glob: None,
            label_glob: Some(Glob::new("test-lala*").unwrap().compile_matcher()),
            index: None,
        };

        // Act
        let found_match = find_match(&otp_database, filter);

        // Assert
        assert!(found_match.is_none());
    }

    #[test]
    fn test_glob_filtering_multiple_filters_match() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer")
                .label("test-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer2")
                .label("test-label2")
                .secret("AA")
                .build()
                .unwrap(),
        );

        let filter = ExtractFilterGlob {
            issuer_glob: Some(Glob::new("test*").unwrap().compile_matcher()),
            label_glob: Some(Glob::new("test-la*").unwrap().compile_matcher()),
            index: None,
        };

        // Act
        let found_match = find_match(&otp_database, filter);

        // Assert
        assert!(found_match.is_some());
    }

    #[test]
    fn test_glob_filtering_multiple_filters_no_match() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer")
                .label("test-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("test-issuer2")
                .label("test-label2")
                .secret("AA")
                .build()
                .unwrap(),
        );

        let filter = ExtractFilterGlob {
            issuer_glob: Some(Glob::new("test-no*").unwrap().compile_matcher()),
            label_glob: Some(Glob::new("test-la*").unwrap().compile_matcher()),
            index: None,
        };

        // Act
        let found_match = find_match(&otp_database, filter);

        // Assert
        assert!(found_match.is_none());
    }
}
