use super::{elem_type::*, node_base::*, xml};

#[derive(Debug, Clone)]
pub struct IntegerNode {
    attr_base: NodeAttributeBase,

    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,

    streamable: bool,

    value_kind: IntegerValueKind,

    min: ImmOrPNode<i64>,

    max: ImmOrPNode<i64>,

    inc: ImmOrPNode<i64>,

    unit: Option<String>,

    representation: IntegerRepresentation,

    p_selected: Vec<String>,
}

impl IntegerNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn value_kind(&self) -> &IntegerValueKind {
        &self.value_kind
    }

    pub fn min(&self) -> &ImmOrPNode<i64> {
        &self.min
    }

    pub fn max(&self) -> &ImmOrPNode<i64> {
        &self.max
    }

    pub fn inc(&self) -> &ImmOrPNode<i64> {
        &self.inc
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_ref().map(|unit| unit.as_ref())
    }

    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }

    pub(super) fn parse(mut node: xml::Node) -> Self {
        debug_assert!(node.tag_name() == "Integer");

        let attr_base = NodeAttributeBase::parse(&node);
        let elem_base = NodeElementBase::parse(&mut node);

        let mut p_invalidators: Vec<String> = vec![];
        while let Some(text) = node.next_text_if("pInvalidator") {
            p_invalidators.push(text.into());
        }

        let streamable = node
            .next_text_if("Streamable")
            .map(|text| convert_to_bool(&text))
            .unwrap_or_default();

        let value_kind = IntegerValueKind::parse(&mut node);

        let min = ImmOrPNode::parse(&mut node, "Min", "pMin");

        let max = ImmOrPNode::parse(&mut node, "Max", "pMax");

        let inc = ImmOrPNode::parse(&mut node, "Inc", "pInc").unwrap_or_else(|| ImmOrPNode::Imm(1));
        let unit = node.next_text_if("Unit").map(Into::into);

        let representation = node
            .next_text_if("Representation")
            .map(|text| text.into())
            .unwrap_or_else(|| IntegerRepresentation::PureNumber);

        // Deduce min and max value based on representation if not specified.
        let min = min.unwrap_or_else(|| ImmOrPNode::imm(representation.deduce_min()));
        let max = max.unwrap_or_else(|| ImmOrPNode::imm(representation.deduce_max()));

        let mut p_selected: Vec<String> = vec![];
        while let Some(text) = node.next_text_if("pSelected") {
            p_selected.push(text.into())
        }

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            value_kind,
            min,
            max,
            inc,
            unit,
            representation,
            p_selected,
        }
    }
}

impl IntegerRepresentation {
    /// Deduce defalut value of min element.
    fn deduce_min(&self) -> i64 {
        use IntegerRepresentation::*;
        match self {
            Linear | Logarithmic | Boolean | PureNumber | HexNumber => i64::MIN,
            IpV4Address | MacAddress => 0,
        }
    }

    /// Deduce defalut value of max element.
    fn deduce_max(&self) -> i64 {
        use IntegerRepresentation::*;
        match self {
            Linear | Logarithmic | Boolean | PureNumber | HexNumber => i64::MAX,
            IpV4Address => 0xffff_ffff,
            MacAddress => 0xffff_ffff_ffff,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntegerValueKind {
    Value(i64),
    PValue(IntegerPValue),
    PIndex(IntegerPIndex),
}

impl IntegerValueKind {
    fn parse(node: &mut xml::Node) -> Self {
        let peek = node.peek().unwrap();
        match peek.tag_name() {
            "Value" => {
                let data = node.next_text_if("Value").unwrap();
                IntegerValueKind::Value(convert_to_int(&data))
            }
            "pValueCopy" | "pValue" => {
                let p_value = IntegerPValue::parse(node);
                IntegerValueKind::PValue(p_value)
            }
            "pIndex" => {
                let p_index = IntegerPIndex::parse(node);
                IntegerValueKind::PIndex(p_index)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntegerPValue {
    pub p_value: String,
    pub p_value_copies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IntegerPIndex {
    pub p_index: String,
    pub value_indexed: Vec<ValueIndexed>,
    pub value_default: ImmOrPNode<i64>,
}

impl IntegerPIndex {
    fn parse(node: &mut xml::Node) -> Self {
        let p_index = node.next_text_if("pIndex").unwrap().into();

        let mut value_indexed = vec![];
        while let Some(value_indexed_elem) = ValueIndexed::parse(node) {
            value_indexed.push(value_indexed_elem);
        }

        let value_default = ImmOrPNode::parse(node, "ValueDefault", "pValueDefault").unwrap();

        Self {
            p_index,
            value_indexed,
            value_default,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueIndexed {
    pub indexed: ImmOrPNode<i64>,
    pub index: i64,
}

impl ValueIndexed {
    fn parse(node: &mut xml::Node) -> Option<Self> {
        if let Some(index) = node.next_if("ValueIndexed") {
            let indexed = ImmOrPNode::imm(convert_to_int(&index.text()));
            let index = convert_to_int(&index.attribute_of("Index").unwrap());
            Some(Self { indexed, index })
        } else if let Some(p_index) = node.next_if("pValueIndexed") {
            let indexed = ImmOrPNode::pnode(p_index.text().into());
            let index = convert_to_int(&p_index.attribute_of("Index").unwrap());
            Some(Self { indexed, index })
        } else {
            None
        }
    }
}

impl IntegerPValue {
    fn parse(node: &mut xml::Node) -> Self {
        // NOTE: The pValue can be sandwiched between two pValueCopy sequence.
        let mut p_value_copies = vec![];
        while let Some(text) = node.next_text_if("pValueCopy") {
            p_value_copies.push(text.into());
        }

        let p_value = node.next_text_if("pValue").unwrap().into();

        while let Some(text) = node.next_text_if("pValueCopy") {
            p_value_copies.push(text.into());
        }

        Self {
            p_value,
            p_value_copies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn integer_node_from_str(xml: &str) -> IntegerNode {
        let document = roxmltree::Document::parse(xml).unwrap();
        let node = xml::Node::from_xmltree_node(document.root_element());

        IntegerNode::parse(node)
    }

    #[test]
    fn test_integer_node_with_immediate() {
        let xml = r#"
            <Integer Name = "TestNode">
                <pInvalidator>Invalidator0</pInvalidator>
                <pInvalidator>Invalidator1</pInvalidator>
                <Streamable>Yes</Streamable>
                <Value>100</Value>
                <Min>10</Min>
                <Max>100</Max>
                <Inc>5</Inc>
                <Unit>dB</Unit>
                <Representation>Logarithmic</Representation>
                <pSelected>Selected0</pSelected>
                <pSelected>Selected1</pSelected>

            </Integer>
            "#;

        let node = integer_node_from_str(xml);

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0], "Invalidator0");
        assert_eq!(p_invalidators[1], "Invalidator1");

        assert!(node.streamable());
        assert!(matches! {node.value_kind(), IntegerValueKind::Value(100)});
        assert_eq!(node.min(), &ImmOrPNode::Imm(10));
        assert_eq!(node.max(), &ImmOrPNode::Imm(100));
        assert_eq!(node.inc(), &ImmOrPNode::Imm(5));
        assert_eq!(node.unit(), Some("dB"));
        assert_eq!(node.representation(), IntegerRepresentation::Logarithmic);

        let p_selected = node.p_selected();
        assert_eq!(p_selected.len(), 2);
        assert_eq!(p_selected[0], "Selected0");
        assert_eq!(p_selected[1], "Selected1");
    }

    #[test]
    fn test_integer_node_with_p_value() {
        let xml = r#"
            <Integer Name = "TestNode">
                <pValueCopy>Copy1</pValueCopy>
                <pValue>pValue</pValue>
                <pValueCopy>Copy2</pValueCopy>
                <pValueCopy>Copy3</pValueCopy>
                <pMin>pMinNode</pMin>
                <pMax>pMaxNode</pMax>
                <pInc>pIncNode</pInc>
            </Integer>
            "#;

        let node = integer_node_from_str(xml);
        let p_value = match node.value_kind() {
            IntegerValueKind::PValue(p_value) => p_value,
            _ => panic!(),
        };
        assert_eq!(p_value.p_value.as_str(), "pValue");
        let p_value_copies = &p_value.p_value_copies;
        assert_eq!(p_value_copies.len(), 3);
        assert_eq!(p_value_copies[0], "Copy1");
        assert_eq!(p_value_copies[1], "Copy2");
        assert_eq!(p_value_copies[2], "Copy3");

        assert_eq!(node.min(), &ImmOrPNode::PNode("pMinNode".into()));
        assert_eq!(node.max(), &ImmOrPNode::PNode("pMaxNode".into()));
        assert_eq!(node.inc(), &ImmOrPNode::PNode("pIncNode".into()));
    }

    #[test]
    fn test_integer_node_with_p_index() {
        let xml = r#"
        <Integer Name="TestNode">
            <pIndex>pIndexNode</pIndex>
            <ValueIndexed Index="10">100</ValueIndexed>
            <pValueIndexed Index="20">pValueIndexNode</pValueIndexed>
            <ValueIndexed Index="30">300</ValueIndexed>
            <pValueDefault>pValueDefaultNode</pValueDefault>
        </Integer>
        "#;

        let node = integer_node_from_str(xml);
        let p_index = match node.value_kind {
            IntegerValueKind::PIndex(p_index) => p_index,
            _ => panic!(),
        };

        assert_eq!(&p_index.p_index, "pIndexNode");

        let value_indexed = p_index.value_indexed;
        assert_eq!(value_indexed.len(), 3);
        assert!(matches! {value_indexed[0].indexed, ImmOrPNode::Imm(100)});
        assert_eq!(value_indexed[0].index, 10);

        assert_eq!(
            value_indexed[1].indexed,
            ImmOrPNode::PNode("pValueIndexNode".into())
        );
        assert_eq!(value_indexed[1].index, 20);

        assert!(matches! {value_indexed[2].indexed, ImmOrPNode::Imm(300)});
        assert_eq!(value_indexed[2].index, 30);

        assert_eq!(
            p_index.value_default,
            ImmOrPNode::PNode("pValueDefaultNode".into())
        );
    }
}
