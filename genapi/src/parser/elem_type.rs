use crate::{
    elem_type::{
        AccessMode, AddressKind, BitMask, CachingMode, DisplayNotation, Endianness,
        FloatRepresentation, ImmOrPNode, IntegerRepresentation, MergePriority, NameSpace,
        NamedValue, PIndex, PValue, RegPIndex, Sign, Slope, StandardNameSpace, ValueIndexed,
        ValueKind, Visibility,
    },
    store::{BooleanId, FloatId, IntegerId, NodeData, NodeId, ValueStore, WritableNodeStore},
    IntSwissKnifeNode,
};

use super::{
    elem_name::{
        ADDRESS, BIT, INDEX, INT_SWISS_KNIFE, NAME, OFFSET, P_ADDRESS, P_INDEX, P_OFFSET, P_VALUE,
        P_VALUE_COPY, P_VALUE_INDEXED, VALUE, VALUE_INDEXED,
    },
    xml, Parse,
};

impl Default for NameSpace {
    fn default() -> Self {
        Self::Custom
    }
}

impl From<&str> for NameSpace {
    fn from(value: &str) -> Self {
        match value {
            "Standard" => Self::Standard,
            "Custom" => Self::Custom,
            _ => unreachable!(),
        }
    }
}

impl Parse for NameSpace {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        node.next_text().unwrap().into()
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Beginner
    }
}

impl From<&str> for Visibility {
    fn from(value: &str) -> Self {
        match value {
            "Beginner" => Self::Beginner,
            "Expert" => Self::Expert,
            "Guru" => Self::Guru,
            "Invisible" => Self::Invisible,
            _ => unreachable!(),
        }
    }
}

impl Parse for Visibility {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        node.next_text().unwrap().into()
    }
}

impl From<&str> for MergePriority {
    fn from(value: &str) -> Self {
        match value {
            "1" => Self::High,
            "0" => Self::Mid,
            "-1" => Self::Low,
            _ => unreachable!(),
        }
    }
}

impl Default for MergePriority {
    fn default() -> Self {
        Self::Mid
    }
}

impl Parse for MergePriority {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        node.next_text().unwrap().into()
    }
}

impl From<&str> for AccessMode {
    fn from(value: &str) -> Self {
        match value {
            "RO" => Self::RO,
            "WO" => Self::WO,
            "RW" => Self::RW,
            _ => unreachable!(),
        }
    }
}

impl Parse for AccessMode {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        node.next_text().unwrap().into()
    }
}

impl Parse for ImmOrPNode<i64> {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let peeked_text = node.peek().unwrap().text();
        if peeked_text.chars().next().unwrap().is_alphabetic() {
            Self::PNode(node.parse(node_store, value_store))
        } else {
            Self::Imm(node.parse(node_store, value_store))
        }
    }
}

impl Parse for ImmOrPNode<f64> {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let peeked_text = node.peek().unwrap().text();

        if peeked_text == "INF"
            || peeked_text == "-INF"
            || peeked_text == "NaN"
            || !peeked_text.chars().next().unwrap().is_alphabetic()
        {
            Self::Imm(node.parse(node_store, value_store))
        } else {
            Self::PNode(node.parse(node_store, value_store))
        }
    }
}

impl Parse for ImmOrPNode<bool> {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let peeked_text = node.peek().unwrap().text();
        if peeked_text == "Yes"
            || peeked_text == "No"
            || peeked_text == "true"
            || peeked_text == "false"
        {
            Self::Imm(node.parse(node_store, value_store))
        } else {
            Self::PNode(node.parse(node_store, value_store))
        }
    }
}

macro_rules! impl_parse_for_imm_or_pnode_id {
    ($id:ty, $value_ty:ty) => {
        impl Parse for ImmOrPNode<$id> {
            fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
            where
                T: WritableNodeStore,
                U: ValueStore,
            {
                let node: ImmOrPNode<$value_ty> = node.parse(node_store, value_store);
                match node {
                    ImmOrPNode::Imm(b) => {
                        let id = value_store.store(b);
                        ImmOrPNode::Imm(id)
                    }
                    ImmOrPNode::PNode(id) => ImmOrPNode::PNode(id),
                }
            }
        }
    };
}

impl_parse_for_imm_or_pnode_id!(IntegerId, i64);
impl_parse_for_imm_or_pnode_id!(FloatId, f64);
impl_parse_for_imm_or_pnode_id!(BooleanId, bool);

impl Default for IntegerRepresentation {
    fn default() -> Self {
        Self::PureNumber
    }
}

impl Parse for IntegerRepresentation {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        use IntegerRepresentation::{
            Boolean, HexNumber, IpV4Address, Linear, Logarithmic, MacAddress, PureNumber,
        };

        let value = node.next_text().unwrap();
        match value {
            "Linear" => Linear,
            "Logarithmic" => Logarithmic,
            "Boolean" => Boolean,
            "PureNumber" => PureNumber,
            "HexNumber" => HexNumber,
            "IPV4Address" => IpV4Address,
            "MACAddress" => MacAddress,
            _ => unreachable!(),
        }
    }
}

impl Parse for FloatRepresentation {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let value = node.next_text().unwrap();
        match value {
            "Linear" => Self::Linear,
            "Logarithmic" => Self::Logarithmic,
            "PureNumber" => Self::PureNumber,
            _ => unreachable!(),
        }
    }
}

impl Default for FloatRepresentation {
    fn default() -> Self {
        Self::PureNumber
    }
}

impl Parse for Slope {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let value = node.next_text().unwrap();
        match value {
            "Increasing" => Self::Increasing,
            "Decreasing" => Self::Decreasing,
            "Varying" => Self::Varying,
            "Automatic" => Self::Automatic,
            _ => unreachable!(),
        }
    }
}

impl Default for Slope {
    fn default() -> Self {
        Self::Automatic
    }
}

impl Default for DisplayNotation {
    fn default() -> Self {
        Self::Automatic
    }
}

impl Parse for DisplayNotation {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let value = node.next_text().unwrap();
        match value {
            "Automatic" => Self::Automatic,
            "Fixed" => Self::Fixed,
            "Scientific" => Self::Scientific,
            _ => unreachable!(),
        }
    }
}

impl From<&str> for StandardNameSpace {
    fn from(value: &str) -> Self {
        match value {
            "None" => Self::None,
            "IIDC" => Self::IIDC,
            "GEV" => Self::GEV,
            "CL" => Self::CL,
            "USB" => Self::USB,
            _ => unreachable!(),
        }
    }
}

impl Default for CachingMode {
    fn default() -> Self {
        Self::WriteThrough
    }
}

impl From<&str> for CachingMode {
    fn from(value: &str) -> Self {
        match value {
            "WriteThrough" => Self::WriteThrough,
            "WriteAround" => Self::WriteAround,
            "NoCache" => Self::NoCache,
            _ => unreachable!(),
        }
    }
}

impl Parse for CachingMode {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let text = node.next_text().unwrap();
        text.into()
    }
}

impl<T> Parse for NamedValue<T>
where
    T: Clone + PartialEq + Parse,
{
    fn parse<U, S>(node: &mut xml::Node, node_store: &mut U, value_store: &mut S) -> Self
    where
        U: WritableNodeStore,
        S: ValueStore,
    {
        let name = node.peek().unwrap().attribute_of(NAME).unwrap().into();
        let value = node.parse(node_store, value_store);
        Self { name, value }
    }
}

pub(super) fn convert_to_bool(value: &str) -> bool {
    match value {
        "Yes" | "true" => true,
        "No" | "false" => false,
        _ => unreachable!(),
    }
}

impl Parse for bool {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let text = node.next_text().unwrap();
        convert_to_bool(text)
    }
}

pub(super) fn convert_to_int(value: &str) -> i64 {
    if value.starts_with("0x") || value.starts_with("0X") {
        i64::from_str_radix(&value[2..], 16).unwrap()
    } else {
        value.parse().unwrap()
    }
}

pub(super) fn convert_to_uint(value: &str) -> u64 {
    if value.starts_with("0x") || value.starts_with("0X") {
        u64::from_str_radix(&value[2..], 16).unwrap()
    } else {
        value.parse().unwrap()
    }
}

impl Parse for i64 {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let value = node.next_text().unwrap();
        convert_to_int(value)
    }
}

impl Parse for u64 {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let value = node.next_text().unwrap();
        convert_to_uint(value)
    }
}

impl Parse for f64 {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
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
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let text = node.next_text().unwrap();
        text.into()
    }
}

impl Parse for NodeId {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let text = node.next_text().unwrap();
        node_store.id_by_name(text)
    }
}

macro_rules! impl_parse_for_value_id {
    ($id:ty, $value_ty:ty) => {
        impl Parse for $id {
            fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
            where
                T: WritableNodeStore,
                U: ValueStore,
            {
                let value: $value_ty = node.parse(node_store, value_store);
                let id = value_store.store(value);
                id
            }
        }
    };
}

impl_parse_for_value_id!(IntegerId, i64);
impl_parse_for_value_id!(FloatId, f64);
impl_parse_for_value_id!(BooleanId, bool);

impl<T> Parse for ValueKind<T>
where
    T: Clone + Parse + PartialEq,
    ImmOrPNode<T>: Parse,
{
    fn parse<U, S>(node: &mut xml::Node, node_store: &mut U, value_store: &mut S) -> Self
    where
        U: WritableNodeStore,
        S: ValueStore,
    {
        let peek = node.peek().unwrap();
        match peek.tag_name() {
            VALUE => ValueKind::Value(node.parse(node_store, value_store)),
            P_VALUE_COPY | P_VALUE => {
                let p_value = node.parse(node_store, value_store);
                ValueKind::PValue(p_value)
            }
            P_INDEX => {
                let p_index = node.parse(node_store, value_store);
                ValueKind::PIndex(p_index)
            }
            _ => unreachable!(),
        }
    }
}

impl Parse for PValue {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        // NOTE: The pValue can be sandwiched between two pValueCopy sequence.
        let mut p_value_copies = node.parse_while(P_VALUE_COPY, node_store, value_store);

        let p_value = node.parse(node_store, value_store);

        let node_ids: Vec<NodeId> = node.parse_while(P_VALUE_COPY, node_store, value_store);
        p_value_copies.extend(node_ids);

        Self {
            p_value,
            p_value_copies,
        }
    }
}

impl<T> Parse for PIndex<T>
where
    T: Clone + PartialEq + Parse,
    ImmOrPNode<T>: Parse,
{
    fn parse<U, S>(node: &mut xml::Node, node_store: &mut U, value_store: &mut S) -> Self
    where
        U: WritableNodeStore,
        S: ValueStore,
    {
        let p_index = node.parse(node_store, value_store);

        let mut value_indexed = vec![];
        while let Some(indexed) = node
            .parse_if(VALUE_INDEXED, node_store, value_store)
            .or_else(|| node.parse_if(P_VALUE_INDEXED, node_store, value_store))
        {
            value_indexed.push(indexed);
        }

        let value_default = node.parse(node_store, value_store);

        Self {
            p_index,
            value_indexed,
            value_default,
        }
    }
}

impl<T> Parse for ValueIndexed<T>
where
    T: Clone + PartialEq + Parse,
    ImmOrPNode<T>: Parse,
{
    fn parse<U, S>(node: &mut xml::Node, node_store: &mut U, value_store: &mut S) -> Self
    where
        U: WritableNodeStore,
        S: ValueStore,
    {
        let index = convert_to_int(node.peek().unwrap().attribute_of(INDEX).unwrap());
        let indexed = node.parse(node_store, value_store);
        Self { index, indexed }
    }
}

impl Parse for AddressKind {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let peeked_node = node.peek().unwrap();
        match peeked_node.tag_name() {
            ADDRESS | P_ADDRESS => Self::Address(node.parse(node_store, value_store)),
            INT_SWISS_KNIFE => {
                let swiss_knife: IntSwissKnifeNode =
                    node.next().unwrap().parse(node_store, value_store);
                let id = swiss_knife.node_base().id();
                node_store.store_node(id, NodeData::IntSwissKnife(swiss_knife.into()));
                Self::IntSwissKnife(id)
            }
            P_INDEX => Self::PIndex(node.parse(node_store, value_store)),
            _ => unreachable!(),
        }
    }
}

impl Parse for RegPIndex {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let next_node = node.peek().unwrap();

        let imm_offset = next_node
            .attribute_of(OFFSET)
            .map(|s| ImmOrPNode::Imm(convert_to_int(s)));
        let pnode_offset = next_node
            .attribute_of(P_OFFSET)
            .map(|s| ImmOrPNode::PNode(node_store.id_by_name(s)));
        let offset = imm_offset.xor(pnode_offset);

        let p_index = node.parse(node_store, value_store);

        Self { offset, p_index }
    }
}

impl Default for Endianness {
    fn default() -> Self {
        Self::LE
    }
}

impl Parse for Endianness {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        match node.next_text().unwrap() {
            "LittleEndian" => Self::LE,
            "BigEndian" => Self::BE,
            _ => unreachable!(),
        }
    }
}

impl Default for Sign {
    fn default() -> Self {
        Self::Unsigned
    }
}

impl Parse for Sign {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        match node.next_text().unwrap() {
            "Signed" => Self::Signed,
            "Unsigned" => Self::Unsigned,
            _ => unreachable!(),
        }
    }
}

impl Parse for BitMask {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        node.parse_if(BIT, node_store, value_store).map_or_else(
            || {
                let lsb = node.parse(node_store, value_store);
                let msb = node.parse(node_store, value_store);
                Self::Range { lsb, msb }
            },
            Self::SingleBit,
        )
    }
}
