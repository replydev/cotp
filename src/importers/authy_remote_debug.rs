//! Import the URI-bearing JSON produced by Authy's remote-debug export script.
use crate::otp::{from_otp_uri::FromOtpUri, otp_element::OTPElement};
use color_eyre::eyre::ErrReport;
use serde::Deserialize;

#[derive(Deserialize)]
struct AuthyExportedJsonElement {
    name: String,
    uri: String,
}

#[derive(Deserialize)]
pub struct AuthyExportedList(Vec<AuthyExportedJsonElement>);

impl TryFrom<AuthyExportedList> for Vec<OTPElement> {
    type Error = ErrReport;
    fn try_from(exported: AuthyExportedList) -> Result<Self, Self::Error> {
        exported
            .0
            .into_iter()
            .map(|entry| {
                let mut element = OTPElement::from_otp_uri(&entry.uri)?;
                element.label = entry.name.clone();
                Ok(element)
            })
            .collect()
    }
}
