//! Account metadata matching shared by the command line and dashboard.
use super::otp_element::OTPElement;

pub fn matches_filters(element: &OTPElement, tags: &[String], favorite: bool) -> bool {
    (!favorite || element.favorite)
        && tags.iter().all(|tag| {
            element
                .tags
                .iter()
                .any(|value| value.eq_ignore_ascii_case(tag))
        })
}

pub fn matches_search(element: &OTPElement, query: &str) -> bool {
    let text = format!(
        "{} {} {} {}",
        element.issuer,
        element.label,
        element.alias.as_deref().unwrap_or(""),
        element.id
    )
    .to_lowercase();
    query.split_whitespace().all(|term| {
        let term = term.to_lowercase();
        if let Some(tag) = term.strip_prefix("tag:") {
            element.tags.iter().any(|value| value.to_lowercase() == tag)
        } else if term == "is:favorite" {
            element.favorite
        } else {
            text.contains(&term)
        }
    })
}

pub fn parse_tag(value: &str) -> Result<String, String> {
    let tag = value.trim();
    if tag.is_empty() || tag.chars().count() > 64 || tag.chars().any(char::is_control) {
        return Err("Tags must contain 1–64 characters without control characters".into());
    }
    Ok(tag.to_owned())
}

pub fn add_tags(existing: &mut Vec<String>, tags: impl IntoIterator<Item = String>) {
    for tag in tags {
        if !existing
            .iter()
            .any(|value| value.eq_ignore_ascii_case(&tag))
        {
            existing.push(tag);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::otp::otp_element::{OTPDatabase, OTPElementBuilder};

    #[test]
    fn explicit_maximum_position_precedes_unordered_and_ties_use_labels_then_ids() {
        let element = |label: &str, id: &str, order: Option<i64>, favorite: bool| {
            OTPElementBuilder::default()
                .secret("AA")
                .issuer("Example")
                .label(label)
                .id(id)
                .order(order)
                .favorite(favorite)
                .build()
                .unwrap()
        };
        let mut db = OTPDatabase::from(vec![
            element("Z", "z", None, true),
            element("B", "b", Some(i64::MAX), false),
            element("A", "b", None, false),
            element("A", "a", None, false),
        ]);
        db.sort();
        let ids: Vec<_> = db
            .elements_ref()
            .iter()
            .map(|element| element.id.as_str())
            .collect();
        assert_eq!(ids, ["b", "a", "b", "z"]);
    }
}
