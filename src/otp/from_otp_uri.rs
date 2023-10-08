use regex::{Captures, Regex};

use super::{otp_algorithm::OTPAlgorithm, otp_element::OTPElement, otp_type::OTPType};

macro_rules! lazy_regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

const DECODE_LAMBDA: fn(&str) -> String = |i| {
    urlencoding::decode(i)
        .map(|i| i.to_string())
        .unwrap_or_else(|i| i.to_string())
};

pub trait FromOtpUri: Sized {
    fn from_otp_uri(otp_uri: &str) -> Result<Self, String>;
}

impl FromOtpUri for OTPElement {
    fn from_otp_uri(otp_uri: &str) -> Result<Self, String> {
        let otp_type = get_match(lazy_regex!(r#"otpauth:[/][/]([a-zA-Z])[/]"#), otp_uri)
            .map(|r| r.to_uppercase())
            .unwrap_or_else(|_| "TOTP".to_string());
        let (issuer, label) = lazy_regex!(r"[a-zA-Z][/](?:(.*):)(.+)\?")
            .captures(otp_uri)
            .map(|c| get_issuer_and_label(c))
            .unwrap_or((None, None));

        if issuer.is_none() {
            return Err(String::from("Issuer not found in OTP Uri"));
        }

        let secret = get_match(lazy_regex!(r#"[?&]secret=(.*?)(?:&|$)"#), otp_uri)?.to_uppercase();
        let algorithm = get_match(lazy_regex!(r#"[?&]algorithm=(.*?)(?:&|$)"#), otp_uri)
            .map(|r| r.to_uppercase())
            .unwrap_or_else(|_| "SHA1".to_string());
        let digits = get_match(lazy_regex!(r"[?&]digits=(\d*?)(?:&|$)"), otp_uri)
            .map(|r| r.parse::<u64>().unwrap())
            .unwrap_or(6);
        let period = get_match(lazy_regex!(r"[?&]period=(\d*?)(?:&|$)"), otp_uri)
            .map(|r| r.parse::<u64>().unwrap())
            .unwrap_or(30);
        let counter = get_match(lazy_regex!(r"[?&]counter=(\d*?)(?:&|$)"), otp_uri)
            .map(|r| Some(r.parse::<u64>().unwrap()))
            .unwrap_or(None);

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

fn get_match(regex: &Regex, value: &str) -> Result<String, String> {
    let optional_value = regex.captures(value);
    if optional_value.is_none() {
        return Err(String::from("No match found"));
    }
    let match_str = optional_value.unwrap().get(1).unwrap();
    Ok(match_str.as_str().to_owned())
}

fn get_issuer_and_label(c: Captures) -> (Option<String>, Option<String>) {
    let issuer = c.get(1).map(|i| i.as_str()).map(DECODE_LAMBDA);
    let label = c.get(2).map(|l| l.as_str()).map(DECODE_LAMBDA);
    (issuer, label)
}
