use color_eyre::eyre::ErrReport;
use url::Url;

use super::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

pub trait FromOtpUri: Sized {
    fn from_otp_uri(otp_uri: &str) -> color_eyre::Result<Self>;
}

impl FromOtpUri for OTPElement {
    fn from_otp_uri(otp_uri: &str) -> color_eyre::Result<Self> {
        let decoded = urlencoding::decode(otp_uri).map_err(ErrReport::from)?;
        let parsed_uri = Url::parse(&decoded).map_err(ErrReport::from)?;

        let otp_type = parsed_uri
            .host_str()
            .map_or_else(|| "TOTP".to_string(), str::to_uppercase);

        let (issuer, label) = get_issuer_and_label(&parsed_uri)?;

        let secret = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "secret")
            .map(|(_k, v)| v.to_uppercase())
            .ok_or(ErrReport::msg("Secret not found in OTP Uri"))?;

        let algorithm = parsed_uri
            .query_pairs()
            .find(|(k, _v)| k == "algorithm")
            .map_or_else(|| "SHA1".to_string(), |(_k, v)| v.to_uppercase());

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
            .and_then(|(_k, v)| v.parse::<u64>().ok());

        Ok(OTPElement {
            secret,
            issuer,
            label,
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
    let first_segment: Vec<String> = parsed_uri
        .path_segments()
        .map(Iterator::collect::<Vec<_>>)
        .ok_or(ErrReport::msg("Failed to collect path segments"))?
        .first()
        .ok_or(ErrReport::msg("No path segments found"))?
        .split(':')
        .collect::<Vec<_>>()
        .into_iter()
        .map(std::borrow::ToOwned::to_owned)
        .collect();
    Ok(first_segment)
}

fn get_issuer_and_label(parsed_uri: &Url) -> color_eyre::Result<(String, String)> {
    // Find the first path segments, OTP Uris should not have others
    let first_segment = get(parsed_uri)?;

    let first = first_segment.first().and_then(|v| {
        urlencoding::decode(v.as_str())
            .map(std::borrow::Cow::into_owned)
            .ok()
    });

    let second = first_segment.get(1).and_then(|v| {
        urlencoding::decode(v)
            .map(std::borrow::Cow::into_owned)
            .ok()
    });

    match (first, second) {
        (Some(i), Some(l)) => Ok((i, l)),
        (Some(l), None) => {
            let issuer = parsed_uri
                .query_pairs()
                .find(|(k, _v)| k == "issuer")
                .map(|(_k, v)| v.to_string())
                .unwrap_or_default();
            Ok((issuer, l))
        }
        _ => Err(ErrReport::msg("No label found in OTP uri")),
    }
}
