use std::convert::TryFrom;

use super::{GenApiError, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSpace {
    Standard,
    Custom,
}

impl Default for NameSpace {
    fn default() -> Self {
        Self::Custom
    }
}

impl TryFrom<Span<&str>> for Span<NameSpace> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> Result<Self, GenApiError> {
        match *value {
            "Standard" => Ok(value.span(NameSpace::Standard)),
            "Custom" => Ok(value.span(NameSpace::Custom)),
            _ => {
                let err_msg = format!("expected Standard or Custom, but {}", *value);
                Err(GenApiError::InvalidData(value.span(err_msg)))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Beginner,
    Expert,
    Guru,
    Invisible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Beginner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePriority {
    High,
    Mid,
    Low,
}

impl Default for MergePriority {
    fn default() -> Self {
        MergePriority::Mid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    RO,
    WO,
    RW,
}
