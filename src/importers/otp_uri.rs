use crate::exporters::otp_uri::OtpUriList;
use crate::otp::from_otp_uri::FromOtpUri;
use crate::otp::otp_element::OTPElement;

impl TryFrom<OtpUriList> for Vec<OTPElement> {
    type Error = String;

    fn try_from(value: OtpUriList) -> Result<Self, Self::Error> {
        Ok(value
            .items
            .into_iter()
            .map(|e| OTPElement::from_otp_uri(e.as_str()))
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .collect())
    }
}
