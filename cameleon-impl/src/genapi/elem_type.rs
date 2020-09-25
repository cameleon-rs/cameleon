use std::convert::{TryFrom, TryInto};

use super::{verifier::verify_node_name, xml::Node, GenApiError, GenApiResult, Span};

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

    fn try_from(value: Span<&str>) -> GenApiResult<Self> {
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

impl TryFrom<Span<&str>> for Span<Visibility> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> GenApiResult<Self> {
        match *value {
            "Beginner" => Ok(value.span(Visibility::Beginner)),
            "Expert" => Ok(value.span(Visibility::Expert)),
            "Guru" => Ok(value.span(Visibility::Guru)),
            "Invisible" => Ok(value.span(Visibility::Invisible)),
            _ => {
                let err_msg = format!(
                    "expected Beginner, Expert, Guru or Invisible, but got '{}'",
                    *value
                );
                Err(GenApiError::InvalidData(value.span(err_msg)))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePriority {
    High,
    Mid,
    Low,
}

impl TryFrom<Span<&str>> for Span<MergePriority> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> GenApiResult<Span<MergePriority>> {
        match *value {
            "1" => Ok(value.span(MergePriority::High)),
            "0" => Ok(value.span(MergePriority::Mid)),
            "-1" => Ok(value.span(MergePriority::Low)),
            _ => {
                let err_msg = format!("expected '1', '0' or '-1', but got '{}'", *value);
                Err(GenApiError::InvalidData(value.span(err_msg)))
            }
        }
    }
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

impl TryFrom<Span<&str>> for Span<AccessMode> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> GenApiResult<Span<AccessMode>> {
        match *value {
            "RO" => Ok(value.span(AccessMode::RO)),
            "WO" => Ok(value.span(AccessMode::WO)),
            "RW" => Ok(value.span(AccessMode::RW)),
            _ => {
                let err_msg = format!("expected 'RO', 'WO' or 'RW', but got '{}'", *value);
                Err(GenApiError::InvalidData(value.span(err_msg)))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImmOrPNode<T: Clone> {
    Imm(Span<T>),
    PNode(Span<String>),
}

impl<T> ImmOrPNode<T>
where
    T: Clone,
{
    pub(super) fn imm(value: Span<T>) -> Self {
        Self::Imm(value)
    }

    pub(super) fn pnode(value: Span<String>) -> Self {
        Self::PNode(value)
    }
}

pub enum IntegerRepresentation {
    Linear,
    Logarithmic,
    Boolean,
    PureNumber,
    HexNumber,
    IpV4Address,
    MacAddress,
}

impl std::convert::TryFrom<Span<&str>> for Span<IntegerRepresentation> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> GenApiResult<Span<IntegerRepresentation>> {
        match *value {
            "Linear" => Ok(value.span(IntegerRepresentation::Linear)),
            "Logarithmic" => Ok(value.span(IntegerRepresentation::Logarithmic)),
            "Boolean" => Ok(value.span(IntegerRepresentation::Boolean)),
            "PureNumber" => Ok(value.span(IntegerRepresentation::PureNumber)),
            "HexNumber" => Ok(value.span(IntegerRepresentation::HexNumber)),
            "IPV4Address" => Ok(value.span(IntegerRepresentation::IpV4Address)),
            "MACAddress" => Ok(value.span(IntegerRepresentation::MacAddress)),
            _ => {
                let error_msg = format!(
                    "expected string of IntRepresentation type, but got '{}",
                    *value
                );
                Err(GenApiError::InvalidData(value.span(error_msg)))
            }
        }
    }
}

impl std::convert::TryFrom<Span<&str>> for Span<bool> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> GenApiResult<Span<bool>> {
        match *value {
            "Yes" => Ok(value.span(true)),
            "No" => Ok(value.span(false)),
            _ => {
                let err_msg = format!("expected 'Yes' or 'No', but got '{}'", *value);
                Err(GenApiError::InvalidData(value.span(err_msg)))
            }
        }
    }
}

impl std::convert::TryFrom<Span<&str>> for Span<i64> {
    type Error = GenApiError;

    fn try_from(value: Span<&str>) -> GenApiResult<Self> {
        let result = if value.starts_with("0x") || value.starts_with("0X") {
            i64::from_str_radix(&value[2..], 16).map(|i| value.span(i))
        } else {
            i64::from_str_radix(*value, 10).map(|i| value.span(i))
        };

        result.map_err(|_| {
            GenApiError::InvalidData(value.span(format!(
                "expected string of hex or decimal number, but got '{}'",
                *value
            )))
        })
    }
}
