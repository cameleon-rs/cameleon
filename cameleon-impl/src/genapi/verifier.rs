use std::ops::Deref;

use lazy_static::lazy_static;
use regex::Regex;

use super::{elem_type::*, GenApiError, GenApiResult, Span};

pub(super) fn verify_node_name(s: Span<&str>) -> GenApiResult<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[A-Za-z][0-9A-Za-z_]*").unwrap();
    }

    if RE.is_match(&s) {
        Ok(())
    } else {
        let err_msg = format!("expected string of node name type, but got {}", s.deref());
        Err(GenApiError::InvalidData(s.span(err_msg)))
    }
}

pub(super) fn verify_hex_string(s: Span<&str>) -> GenApiResult<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([0-9A-Fa-f]){2,}$").unwrap();
    }

    if RE.is_match(&s) {
        Ok(())
    } else {
        let err_msg = format!("expected string of hex type, but got {}", s.deref());
        Err(GenApiError::InvalidData(s.span(err_msg)))
    }
}

pub(super) fn verify_url_string(s: Span<&str>) -> GenApiResult<()> {
    if s.starts_with("http://") {
        Ok(())
    } else {
        let err_msg = format!("expected string of url type, but got {}", s.deref());
        Err(GenApiError::InvalidData(s.span(err_msg)))
    }
}

pub(super) fn convert_to_bool(s: Span<&str>) -> GenApiResult<Span<bool>> {
    match *s {
        "Yes" => Ok(s.span(true)),
        "No" => Ok(s.span(false)),
        _ => {
            let err_msg = format!("expected 'Yes' or 'No', but got '{}'", s.deref());
            Err(GenApiError::InvalidData(s.span(err_msg)))
        }
    }
}

pub(super) fn convert_to_namespace(s: Span<&str>) -> GenApiResult<Span<NameSpace>> {
    match *s {
        "Standard" => Ok(s.span(NameSpace::Standard)),
        "Custom" => Ok(s.span(NameSpace::Custom)),
        _ => {
            let err_msg = format!("expected Standard or Custom, but {}", *s);
            Err(GenApiError::InvalidData(s.span(err_msg)))
        }
    }
}

pub(super) fn convert_to_visibility(s: Span<&str>) -> GenApiResult<Span<Visibility>> {
    match *s {
        "Beginner" => Ok(s.span(Visibility::Beginner)),
        "Expert" => Ok(s.span(Visibility::Expert)),
        "Guru" => Ok(s.span(Visibility::Guru)),
        "Invisible" => Ok(s.span(Visibility::Invisible)),
        _ => {
            let err_msg = format!("expected Beginner, Expert, Guru or Invisible, but {}", *s);
            Err(GenApiError::InvalidData(s.span(err_msg)))
        }
    }
}

pub(super) fn convert_to_access_mode(s: Span<&str>) -> GenApiResult<Span<AccessMode>> {
    match *s {
        "RO" => Ok(s.span(AccessMode::RO)),
        "WO" => Ok(s.span(AccessMode::WO)),
        "RW" => Ok(s.span(AccessMode::RW)),
        _ => {
            let err_msg = format!("expected RO, WO or RW, but  {}", *s);
            Err(GenApiError::InvalidData(s.span(err_msg)))
        }
    }
}

pub(super) fn convert_to_merge_priority(s: Span<&str>) -> GenApiResult<Span<MergePriority>> {
    match *s {
        "1" => Ok(s.span(MergePriority::High)),
        "0" => Ok(s.span(MergePriority::Mid)),
        "-1" => Ok(s.span(MergePriority::Low)),
        _ => {
            let err_msg = format!("expected 1, 0 or -1, but  {}", *s);
            Err(GenApiError::InvalidData(s.span(err_msg)))
        }
    }
}
