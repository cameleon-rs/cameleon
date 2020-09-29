use super::{xml, Parse};

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

impl From<&str> for NameSpace {
    fn from(value: &str) -> Self {
        match value {
            "Standard" => NameSpace::Standard,
            "Custom" => NameSpace::Custom,
            _ => unreachable!(),
        }
    }
}

impl Parse for NameSpace {
    fn parse(node: &mut xml::Node) -> Self {
        node.next_text().unwrap().into()
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

impl From<&str> for Visibility {
    fn from(value: &str) -> Self {
        match value {
            "Beginner" => Visibility::Beginner,
            "Expert" => Visibility::Expert,
            "Guru" => Visibility::Guru,
            "Invisible" => Visibility::Invisible,
            _ => unreachable!(),
        }
    }
}

impl Parse for Visibility {
    fn parse(node: &mut xml::Node) -> Self {
        node.next_text().unwrap().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePriority {
    High,
    Mid,
    Low,
}

impl From<&str> for MergePriority {
    fn from(value: &str) -> Self {
        match value {
            "1" => MergePriority::High,
            "0" => MergePriority::Mid,
            "-1" => MergePriority::Low,
            _ => unreachable!(),
        }
    }
}

impl Default for MergePriority {
    fn default() -> Self {
        MergePriority::Mid
    }
}

impl Parse for MergePriority {
    fn parse(node: &mut xml::Node) -> Self {
        node.next_text().unwrap().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    RO,
    WO,
    RW,
}

impl From<&str> for AccessMode {
    fn from(value: &str) -> Self {
        match value {
            "RO" => AccessMode::RO,
            "WO" => AccessMode::WO,
            "RW" => AccessMode::RW,
            _ => unreachable!(),
        }
    }
}

impl Parse for AccessMode {
    fn parse(node: &mut xml::Node) -> Self {
        node.next_text().unwrap().into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmOrPNode<T: Clone + PartialEq> {
    Imm(T),
    PNode(String),
}

impl<T> Parse for ImmOrPNode<T>
where
    T: Clone + PartialEq + Parse,
{
    fn parse(node: &mut xml::Node) -> Self {
        let next_node = node.peek().unwrap();
        if next_node.text().chars().next().unwrap().is_alphabetic() {
            ImmOrPNode::PNode(node.parse())
        } else {
            ImmOrPNode::Imm(node.parse())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerRepresentation {
    Linear,
    Logarithmic,
    Boolean,
    PureNumber,
    HexNumber,
    IpV4Address,
    MacAddress,
}

impl Parse for IntegerRepresentation {
    fn parse(node: &mut xml::Node) -> Self {
        let value = node.next_text().unwrap();
        match value {
            "Linear" => IntegerRepresentation::Linear,
            "Logarithmic" => IntegerRepresentation::Logarithmic,
            "Boolean" => IntegerRepresentation::Boolean,
            "PureNumber" => IntegerRepresentation::PureNumber,
            "HexNumber" => IntegerRepresentation::HexNumber,
            "IPV4Address" => IntegerRepresentation::IpV4Address,
            "MACAddress" => IntegerRepresentation::MacAddress,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardNameSpace {
    None,
    IIDC,
    GEV,
    CL,
    USB,
}

impl From<&str> for StandardNameSpace {
    fn from(value: &str) -> Self {
        match value {
            "None" => StandardNameSpace::None,
            "IIDC" => StandardNameSpace::IIDC,
            "GEV" => StandardNameSpace::GEV,
            "CL" => StandardNameSpace::CL,
            "USB" => StandardNameSpace::USB,
            _ => unreachable!(),
        }
    }
}

pub(super) fn convert_to_bool(value: &str) -> bool {
    match value {
        "Yes" => true,
        "No" => false,
        _ => unreachable!(),
    }
}

impl Parse for bool {
    fn parse(node: &mut xml::Node) -> Self {
        let text = node.next_text().unwrap();
        convert_to_bool(text)
    }
}

pub(super) fn convert_to_int(value: &str) -> i64 {
    if value.starts_with("0x") || value.starts_with("0X") {
        i64::from_str_radix(&value[2..], 16).unwrap()
    } else {
        i64::from_str_radix(value, 10).unwrap()
    }
}

impl Parse for i64 {
    fn parse(node: &mut xml::Node) -> Self {
        let value = node.next_text().unwrap();
        convert_to_int(value)
    }
}

impl Parse for String {
    fn parse(node: &mut xml::Node) -> Self {
        let text = node.next_text().unwrap();
        text.into()
    }
}
