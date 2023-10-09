use color_eyre::eyre::ErrReport;
use url::Url;

use super::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

pub trait FromOtpUri: Sized {
    fn from_otp_uri(otp_uri: &str) -> Result<Self, String>;
}

impl FromOtpUri for OTPElement {
    fn from_otp_uri(otp_uri: &str) -> Result<Self, String> {
        let parsed_uri = Url::parse(otp_uri).map_err(|e| e.to_string())?;

        let otp_type = parsed_uri
            .host_str()
            .map(|r| r.to_uppercase())
            .unwrap_or_else(|| "TOTP".to_string());

        let (issuer, label) = get_issuer_and_label(&parsed_uri);

        if issuer.is_none() {
            return Err(String::from("Issuer not found in OTP Uri"));
        }

        let secret = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "secret")
            .map(|(_k, v)| v.to_uppercase())
            .ok_or(String::from("Secret not found in OTP Uri"))?;

        let algorithm = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "algorithm")
            .map(|(_k, v)| v.to_uppercase())
            .unwrap_or_else(|| "SHA1".to_string());

        let digits = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "digits")
            .map_or(6, |(_k, v)| v.parse::<u64>().unwrap_or(6));

        let period = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "period")
            .map_or(30, |(_k, v)| v.parse::<u64>().unwrap_or(30));

        let counter = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "counter")
            .map_or(None, |(_k, v)| v.parse::<u64>().ok());

        Ok(OTPElement {
            secret,
            issuer: issuer.unwrap(), // Safe to wrap due to upper check
            label: label.unwrap_or_default(),
            digits,
            type_: OTPType::from(otp_type.as_str()),
            algorithm: OTPAlgorithm::from(algorithm.as_str()),
            period,
            counter,
            pin: None,
        })
    }
}

fn get(parsed_uri: &Url) -> color_eyre::Result<Vec<String>> {
    let mut first_segment: Vec<String> = parsed_uri
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .ok_or(ErrReport::msg("Failed to collect path segments"))?
        .get(0)
        .ok_or(ErrReport::msg("No path segments found"))?
        .split(':')
        .collect::<Vec<_>>()
        .into_iter()
        .map(|v| v.to_owned())
        .collect();
    Ok(first_segment)
}

fn get_issuer_and_label(parsed_uri: &Url) -> (Option<String>, Option<String>) {
    // Find the first path segments, OTP Uris should not have others
    let mut first_segment = get(parsed_uri);
    if first_segment.is_err() {
        return (None, None);
    }

    let issuer: Option<String>;
    let label: Option<String>;
    let mut unwrapped = first_segment.unwrap();
    if unwrapped.len() == 2 {
        issuer = Some(unwrapped.remove(0));
        label = Some(unwrapped.remove(0));
    } else {
        label = Some(unwrapped.remove(0));
        issuer = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "issuer")
            .map(|(_k, v)| v.to_string());
    }

    (issuer, label)
}
