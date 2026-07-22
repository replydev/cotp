use crate::otp::otp_element::OTPDatabase;
use crate::{clipboard, otp::otp_element::OTPElement};
use clap::Args;
use eyre::eyre;

use super::SubcommandExecutor;

#[derive(Args, Default)]
pub struct ExtractArgs {
    /// Code Index
    #[arg(short, long, required_unless_present_any = ["issuer", "label"])]
    pub index: Option<usize>,

    /// Code issuer, may be a wildcard pattern (`*` and `?`)
    #[arg(short = 's', long, required_unless_present_any = ["index", "label"])]
    pub issuer: Option<String>,

    /// Code label, may be a wildcard pattern (`*` and `?`)
    #[arg(short, long, required_unless_present_any = ["index", "issuer"])]
    pub label: Option<String>,

    /// Copy the code to the clipboard
    #[arg(short, long = "copy-clipboard", default_value_t = false)]
    pub copy_to_clipboard: bool,
}

// Contains wildcard filters for each field we can filter on
struct ExtractFilter {
    issuer_pattern: Option<String>,
    label_pattern: Option<String>,
    index: Option<usize>,
}

impl TryFrom<ExtractArgs> for ExtractFilter {
    type Error = eyre::ErrReport;

    fn try_from(value: ExtractArgs) -> Result<Self, Self::Error> {
        if value.index == Some(0) {
            return Err(eyre!(
                "Invalid index 0: indexes are 1-based, use --index 1 for the first code"
            ));
        }

        Ok(Self {
            issuer_pattern: value.issuer,
            label_pattern: value.label,
            index: value.index,
        })
    }
}

/// Case-insensitive wildcard matching of the whole `text` against `pattern`,
/// where `*` matches any (possibly empty) sequence of characters and `?`
/// matches exactly one character.
///
/// This replaces the former globset-based matcher, which pulled the whole
/// regex engine into the binary for this simple use case.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.to_lowercase().chars().collect();
    let text: Vec<char> = text.to_lowercase().chars().collect();

    // Iterative two-pointer matching with backtracking to the last `*`
    let mut p = 0; // position in pattern
    let mut t = 0; // position in text
    let mut star: Option<usize> = None; // position of the last `*` seen
    let mut star_t = 0; // position in text when the last `*` was seen

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            star_t = t;
            p += 1;
        } else if let Some(star_p) = star {
            // Backtrack: let the last `*` consume one more character
            p = star_p + 1;
            star_t += 1;
            t = star_t;
        } else {
            return false;
        }
    }

    // Only trailing `*`s may remain in the pattern
    pattern[p..].iter().all(|&c| c == '*')
}

impl SubcommandExecutor for ExtractArgs {
    fn run_command(self, otp_database: OTPDatabase) -> eyre::Result<OTPDatabase> {
        let copy_to_clipboard = self.copy_to_clipboard;
        let filter: ExtractFilter = self.try_into()?;

        let first_with_filters = find_match(&otp_database, filter);

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

fn find_match(otp_database: &OTPDatabase, filter: ExtractFilter) -> Option<&OTPElement> {
    otp_database
        .elements
        .iter()
        .enumerate()
        .find(|(index, code)| filter_extract(&filter, *index, code))
        .map(|(_, code)| code)
}

fn filter_extract(args: &ExtractFilter, index: usize, candidate: &OTPElement) -> bool {
    // The user-facing index is 1-based (like list, edit, delete and the TUI),
    // while `index` here is the 0-based position in the database
    let match_by_index = args.index.is_none_or(|i| i.checked_sub(1) == Some(index));

    let match_by_issuer = args
        .issuer_pattern
        .as_ref()
        .is_none_or(|issuer| wildcard_match(issuer, &candidate.issuer));

    let match_by_label = args
        .label_pattern
        .as_ref()
        .is_none_or(|label| wildcard_match(label, &candidate.label));

    match_by_index && match_by_issuer && match_by_label
}

#[cfg(test)]
mod tests {

    use crate::{
        arguments::extract::ExtractArgs,
        otp::otp_element::{OTPDatabase, OTPElementBuilder},
    };

    use super::{find_match, wildcard_match};

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
        let result: Result<super::ExtractFilter, _> = filter.try_into();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_wildcard_match_literal() {
        assert!(wildcard_match("test", "test"));
        assert!(!wildcard_match("test", "test2"));
        assert!(!wildcard_match("test", "tes"));
    }

    #[test]
    fn test_wildcard_match_empty_pattern() {
        assert!(wildcard_match("", ""));
        assert!(!wildcard_match("", "a"));
    }

    #[test]
    fn test_wildcard_match_star() {
        assert!(wildcard_match("*", ""));
        assert!(wildcard_match("*", "anything"));
        assert!(wildcard_match("test-*", "test-issuer"));
        assert!(wildcard_match("*issuer", "test-issuer"));
        assert!(wildcard_match("t*t*r", "test-issuer"));
        assert!(!wildcard_match("t*t*z", "test-issuer"));
    }

    #[test]
    fn test_wildcard_match_star_collapse() {
        assert!(wildcard_match("***", "anything"));
        assert!(wildcard_match("a**b", "ab"));
        assert!(wildcard_match("a**b", "a-whatever-b"));
        assert!(!wildcard_match("a**b", "a-whatever-c"));
    }

    #[test]
    fn test_wildcard_match_question_mark() {
        assert!(wildcard_match("?", "a"));
        assert!(!wildcard_match("?", ""));
        assert!(!wildcard_match("?", "ab"));
        assert!(wildcard_match("te?t", "test"));
        assert!(!wildcard_match("te?t", "tet"));
        assert!(wildcard_match("?*", "abc"));
    }

    #[test]
    fn test_wildcard_match_case_insensitive() {
        assert!(wildcard_match("TeSt-iSS*", "test-issuer"));
        assert!(wildcard_match("test", "TEST"));
    }

    #[test]
    fn test_wildcard_match_unicode_case() {
        assert!(wildcard_match("über*", "ÜBERtest"));
        assert!(wildcard_match("ÜBER*", "übertest"));
        assert!(wildcard_match("caf?", "CAFÉ"));
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
