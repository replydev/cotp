use crate::otp::otp_element::OTPDatabase;
use crate::{clipboard, otp::otp_element::OTPElement};
use clap::Args;
use color_eyre::eyre::eyre;
use globset::{GlobBuilder, GlobMatcher};

use super::SubcommandExecutor;

#[derive(Args, Default)]
pub struct ExtractArgs {
    /// Code Index
    #[arg(short, long, required_unless_present_any = ["issuer", "label"])]
    pub index: Option<usize>,

    /// Code issuer, may be a glob pattern
    #[arg(short = 's', long, required_unless_present_any = ["index", "label"])]
    pub issuer: Option<String>,

    /// Code label, may be a glob pattern
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
        if value.index == Some(0) {
            return Err(eyre!(
                "Invalid index 0: indexes are 1-based, use --index 1 for the first code"
            ));
        }

        let issuer_glob = if let Some(issuer) = value.issuer {
            Some(create_matcher(&issuer)?)
        } else {
            None
        };

        let label_glob = if let Some(label) = value.label {
            Some(create_matcher(&label)?)
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

fn create_matcher(
    glob: &str,
) -> Result<GlobMatcher, <ExtractFilterGlob as TryFrom<ExtractArgs>>::Error> {
    Ok(GlobBuilder::new(glob)
        .case_insensitive(true)
        .build()?
        .compile_matcher())
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
    // The user-facing index is 1-based (like list, edit, delete and the TUI),
    // while `index` here is the 0-based position in the database
    let match_by_index = args.index.is_none_or(|i| i.checked_sub(1) == Some(index));

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

    use crate::{
        arguments::extract::ExtractArgs,
        otp::otp_element::{OTPDatabase, OTPElementBuilder},
    };

    use super::find_match;

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

        let filter = ExtractArgs {
            issuer: Some("test-iss*".to_string()),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

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

        let filter = ExtractArgs {
            label: Some("test-la*".to_string()),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

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

        let filter = ExtractArgs {
            label: Some("test-lala*".to_string()),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

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

        let filter = ExtractArgs {
            issuer: Some("test*".to_string()),
            label: Some("test-la*".to_string()),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

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

        let filter = ExtractArgs {
            issuer: Some("test-no*".to_string()),
            label: Some("test-la*".to_string()),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

        // Assert
        assert!(found_match.is_none());
    }

    #[test]
    fn test_index_filtering_is_one_based() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("first-issuer")
                .label("first-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("second-issuer")
                .label("second-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        // Act / Assert: --index 1 must return the FIRST element
        let filter = ExtractArgs {
            index: Some(1),
            ..Default::default()
        };
        let found_match = find_match(&otp_database, filter.try_into().unwrap());
        assert_eq!("first-issuer", found_match.unwrap().issuer);

        // Act / Assert: --index 2 must return the SECOND element
        let filter = ExtractArgs {
            index: Some(2),
            ..Default::default()
        };
        let found_match = find_match(&otp_database, filter.try_into().unwrap());
        assert_eq!("second-issuer", found_match.unwrap().issuer);
    }

    #[test]
    fn test_index_out_of_range_matches_nothing() {
        // Arrange
        let mut otp_database = OTPDatabase::default();
        otp_database.add_element(
            OTPElementBuilder::default()
                .issuer("first-issuer")
                .label("first-label")
                .secret("AA")
                .build()
                .unwrap(),
        );

        let filter = ExtractArgs {
            index: Some(2),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

        // Assert
        assert!(found_match.is_none());
    }

    #[test]
    fn test_index_zero_is_rejected() {
        // Arrange
        let filter = ExtractArgs {
            index: Some(0),
            ..Default::default()
        };

        // Act
        let result: Result<super::ExtractFilterGlob, _> = filter.try_into();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_filtering_case_insensitive() {
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

        let filter = ExtractArgs {
            issuer: Some("TeSt-iSS*".to_string()),
            ..Default::default()
        };

        // Act
        let found_match = find_match(&otp_database, filter.try_into().unwrap());

        // Assert
        assert!(found_match.is_some());
    }
}
