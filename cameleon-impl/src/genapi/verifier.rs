use lazy_static::lazy_static;
use regex::Regex;

use super::{GenApiError, GenApiResult, Span};

pub(super) fn verify_node_name(value: Span<&str>) -> GenApiResult<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[A-Za-z][0-9A-Za-z_]*").unwrap();
    }

    if RE.is_match(&value) {
        Ok(())
    } else {
        let err_msg = format!("expected string of node name type, but got '{}'", *value);
        Err(GenApiError::InvalidData(value.span(err_msg)))
    }
}

pub(super) fn verify_hex_string(value: Span<&str>) -> GenApiResult<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([0-9A-Fa-f]){2,}$").unwrap();
    }

    if RE.is_match(&value) {
        Ok(())
    } else {
        let err_msg = format!("expected string of hex type, but got '{}'", *value);
        Err(GenApiError::InvalidData(value.span(err_msg)))
    }
}

pub(super) fn verify_url_string(value: Span<&str>) -> GenApiResult<()> {
    if value.starts_with("http://") {
        Ok(())
    } else {
        let err_msg = format!("expected string of url type, but got '{}'", *value);
        Err(GenApiError::InvalidData(value.span(err_msg)))
    }
}

pub(super) fn verify_positive_int(value: Span<i64>) -> GenApiResult<()> {
    if *value > 0 {
        Ok(())
    } else {
        Err(GenApiError::InvalidData(value.span(format!(
            "expected positive integer, but got '{}'",
            *value
        ))))
    }
}
