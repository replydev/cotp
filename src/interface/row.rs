use crate::otp::{otp_element::OTPElement, otp_type::OTPType};

pub(crate) struct Row {
    /// Stable database position, independent of the filtered table position.
    pub(crate) database_index: usize,
    pub(crate) issuer: String,
    pub(crate) label: String,
    pub(crate) code: String,
    pub(crate) has_error: bool,
}

impl Row {
    pub(crate) fn new(database_index: usize, element: &OTPElement) -> Self {
        let result = code_for(element);
        Self {
            database_index,
            issuer: element.issuer.clone(),
            label: element.label.clone(),
            has_error: result.is_err(),
            code: result.unwrap_or_else(|error| error),
        }
    }
}

pub(crate) fn code_for(element: &OTPElement) -> Result<String, String> {
    if period_for(element) == Some(0) {
        return Err("Invalid period: must be greater than zero".into());
    }
    element.get_otp_code().map_err(|error| error.to_string())
}

pub(crate) fn period_for(element: &OTPElement) -> Option<u64> {
    match element.type_ {
        OTPType::Hotp => None,
        OTPType::Steam => Some(30),
        _ => Some(element.period),
    }
}

pub(crate) fn remaining(element: &OTPElement, now: u64) -> Option<(u64, u64)> {
    period_for(element)
        .filter(|period| *period > 0)
        .map(|period| (period - now % period, period))
}

pub(crate) fn display_code(code: &str, hidden: bool) -> String {
    if hidden {
        return "••• •••".into();
    }
    if code.len() >= 6 && code.bytes().all(|byte| byte.is_ascii_digit()) {
        let (left, right) = code.split_at(code.len() / 2);
        format!("{left} {right}")
    } else {
        code.to_owned()
    }
}
