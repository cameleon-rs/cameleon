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

impl<T> ImmOrPNode<T>
where
    T: Clone + PartialEq,
{
    pub fn imm(&self) -> Option<&T> {
        match self {
            ImmOrPNode::Imm(value) => Some(value),
            _ => None,
        }
    }

    pub fn pnode(&self) -> Option<&str> {
        match self {
            ImmOrPNode::PNode(node) => Some(node),
            _ => None,
        }
    }
}

impl Parse for ImmOrPNode<i64> {
    fn parse(node: &mut xml::Node) -> Self {
        let next_node = node.peek().unwrap();
        if next_node.text().chars().next().unwrap().is_alphabetic() {
            ImmOrPNode::PNode(node.parse())
        } else {
            ImmOrPNode::Imm(node.parse())
        }
    }
}

impl Parse for ImmOrPNode<f64> {
    fn parse(node: &mut xml::Node) -> Self {
        let next_node = node.peek().unwrap();
        let next_text = next_node.text();

        if next_text == "INF" || next_text == "-INF" || next_text == "NaN" {
            ImmOrPNode::Imm(node.parse())
        } else if next_node.text().chars().next().unwrap().is_alphabetic() {
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

impl IntegerRepresentation {
    /// Deduce defalut value of min element.
    pub(super) fn deduce_min(&self) -> i64 {
        use IntegerRepresentation::*;
        match self {
            Linear | Logarithmic | Boolean | PureNumber | HexNumber => i64::MIN,
            IpV4Address | MacAddress => 0,
        }
    }

    /// Deduce defalut value of max element.
    pub(super) fn deduce_max(&self) -> i64 {
        use IntegerRepresentation::*;
        match self {
            Linear | Logarithmic | Boolean | PureNumber | HexNumber => i64::MAX,
            IpV4Address => 0xffff_ffff,
            MacAddress => 0xffff_ffff_ffff,
        }
    }
}

impl Default for IntegerRepresentation {
    fn default() -> Self {
        Self::PureNumber
    }
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
pub enum FloatRepresentation {
    Linear,
    Logarithmic,
    PureNumber,
}

impl Parse for FloatRepresentation {
    fn parse(node: &mut xml::Node) -> Self {
        let value = node.next_text().unwrap();
        match value {
            "Linear" => FloatRepresentation::Linear,
            "Logarithmic" => FloatRepresentation::Logarithmic,
            "PureNumber" => FloatRepresentation::PureNumber,
            _ => unreachable!(),
        }
    }
}

impl Default for FloatRepresentation {
    fn default() -> Self {
        Self::PureNumber
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayNotation {
    Automatic,
    Fixed,
    Scientific,
}

impl Default for DisplayNotation {
    fn default() -> Self {
        DisplayNotation::Automatic
    }
}

impl Parse for DisplayNotation {
    fn parse(node: &mut xml::Node) -> Self {
        let value = node.next_text().unwrap();
        match value {
            "Automatic" => DisplayNotation::Automatic,
            "Fixed" => DisplayNotation::Fixed,
            "Scientific" => DisplayNotation::Scientific,
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

#[derive(Debug, Clone)]
pub enum ValueKind<T>
where
    T: Clone + PartialEq,
{
    Value(T),
    PValue(PValue),
    PIndex(PIndex<T>),
}

impl<T> Parse for ValueKind<T>
where
    T: Clone + Parse + PartialEq,
    ImmOrPNode<T>: Parse,
{
    fn parse(node: &mut xml::Node) -> Self {
        let peek = node.peek().unwrap();
        match peek.tag_name() {
            "Value" => ValueKind::Value(node.parse()),
            "pValueCopy" | "pValue" => {
                let p_value = node.parse();
                ValueKind::PValue(p_value)
            }
            "pIndex" => {
                let p_index = node.parse();
                ValueKind::PIndex(p_index)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PValue {
    pub p_value: String,
    pub p_value_copies: Vec<String>,
}

impl Parse for PValue {
    fn parse(node: &mut xml::Node) -> Self {
        // NOTE: The pValue can be sandwiched between two pValueCopy sequence.
        let mut p_value_copies = vec![];
        while let Some(copy) = node.parse_if("pValueCopy") {
            p_value_copies.push(copy);
        }

        let p_value = node.parse();

        while let Some(copy) = node.parse_if("pValueCopy") {
            p_value_copies.push(copy);
        }

        Self {
            p_value,
            p_value_copies,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PIndex<T>
where
    T: Clone + PartialEq,
{
    pub p_index: String,
    pub value_indexed: Vec<ValueIndexed<T>>,
    pub value_default: ImmOrPNode<T>,
}

impl<T> Parse for PIndex<T>
where
    T: Clone + PartialEq + Parse,
    ImmOrPNode<T>: Parse,
{
    fn parse(node: &mut xml::Node) -> Self {
        let p_index = node.parse();

        let mut value_indexed = vec![];
        while node.is_next_node_name("ValueIndexed") || node.is_next_node_name("pValueIndexed") {
            value_indexed.push(node.parse());
        }

        let value_default = node.parse();

        Self {
            p_index,
            value_indexed,
            value_default,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueIndexed<T>
where
    T: Clone + PartialEq,
{
    pub index: i64,
    pub indexed: ImmOrPNode<T>,
}

impl<T> Parse for ValueIndexed<T>
where
    T: Clone + PartialEq + Parse,
    ImmOrPNode<T>: Parse,
{
    fn parse(node: &mut xml::Node) -> Self {
        let index = convert_to_int(node.peek().unwrap().attribute_of("Index").unwrap());
        let indexed = node.parse();
        Self { index, indexed }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedValue<T>
where
    T: Clone + PartialEq,
{
    name: String,
    value: T,
}

impl<T> NamedValue<T>
where
    T: Clone + PartialEq,
{
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> Parse for NamedValue<T>
where
    T: Clone + PartialEq + Parse,
{
    fn parse(node: &mut xml::Node) -> Self {
        let name = node.peek().unwrap().attribute_of("Name").unwrap().into();
        let value = node.parse();
        Self { name, value }
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

impl Parse for f64 {
    fn parse(node: &mut xml::Node) -> Self {
        let value = node.next_text().unwrap();
        if value == "INF" {
            f64::INFINITY
        } else if value == "-INF" {
            f64::NEG_INFINITY
        } else {
            value.parse().unwrap()
        }
    }
}

impl Parse for String {
    fn parse(node: &mut xml::Node) -> Self {
        let text = node.next_text().unwrap();
        text.into()
    }
}
