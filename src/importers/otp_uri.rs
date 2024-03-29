use crate::exporters::otp_uri::OtpUriList;
use crate::otp::from_otp_uri::FromOtpUri;
use crate::otp::otp_element::OTPElement;
use color_eyre::eyre::ErrReport;

impl TryFrom<OtpUriList> for Vec<OTPElement> {
    type Error = ErrReport;

    fn try_from(value: OtpUriList) -> Result<Self, Self::Error> {
        value
            .items
            .into_iter()
            .map(|e| OTPElement::from_otp_uri(e.as_str()))
            .collect()
    }
}
