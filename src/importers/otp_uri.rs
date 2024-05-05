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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        exporters::otp_uri::OtpUriList, importers::importer::import_from_path,
        otp::otp_element::OTPElement,
    };

    #[test]
    fn test_conversion() {
        //Arrange
        let expected_element = OTPElement {
            secret: String::from("AA"),
            issuer: String::default(),
            label: String::from("test"),
            digits: 6,
            type_: crate::otp::otp_type::OTPType::Totp,
            algorithm: crate::otp::otp_algorithm::OTPAlgorithm::Sha1,
            period: 30,
            counter: None,
            pin: None,
        };

        // Act
        let mut imported = import_from_path::<OtpUriList>(PathBuf::from(
            "test_samples/otp_uri/input_otp_uri.json",
        ))
        .unwrap();

        // Assert
        assert_eq!(1, imported.len());
        assert_eq!(expected_element, imported.pop().unwrap())
    }
}
