use color_eyre::eyre::{ErrReport, Result};
use data_encoding::BASE32_NOPAD;

use crate::{
    importers::freeotp_plus::{FreeOTPElement, FreeOTPPlusJson},
    otp::otp_element::{OTPDatabase, OTPElement},
};

impl TryFrom<&OTPDatabase> for FreeOTPPlusJson {
    type Error = ErrReport;
    fn try_from(otp_database: &OTPDatabase) -> Result<Self, Self::Error> {
        otp_database
            .elements
            .iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<FreeOTPElement>, ErrReport>>()
            .map(FreeOTPPlusJson::new)
    }
}

impl TryFrom<&OTPElement> for FreeOTPElement {
    type Error = ErrReport;
    fn try_from(otp_element: &OTPElement) -> Result<Self, Self::Error> {
        Ok(FreeOTPElement {
            secret: decode_secret(otp_element.secret.clone())?,
            algo: otp_element.algorithm.to_string(),
            counter: otp_element.counter.unwrap_or(0),
            digits: otp_element.digits,
            issuer_ext: otp_element.issuer.clone(),
            _label: otp_element.label.clone(),
            period: otp_element.period,
            _type: otp_element.type_.to_string(),
        })
    }
}

fn decode_secret(secret: String) -> Result<Vec<i8>> {
    BASE32_NOPAD
        .decode(secret.as_bytes())
        .map(|v| v.into_iter().map(|n| n as i8).collect::<Vec<i8>>())
        .map_err(ErrReport::from)
}
