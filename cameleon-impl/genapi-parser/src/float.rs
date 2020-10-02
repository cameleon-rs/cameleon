use super::{elem_name::*, elem_type::*, node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct FloatNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,
    streamable: bool,
    value_kind: numeric_node_elem::ValueKind<f64>,
    min: ImmOrPNode<f64>,
    max: ImmOrPNode<f64>,
    inc: Option<ImmOrPNode<f64>>,
    unit: Option<String>,
    representation: FloatRepresentation,
    display_notation: DisplayNotation,
    display_precision: i64,
}

impl FloatNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn value_kind(&self) -> &numeric_node_elem::ValueKind<f64> {
        &self.value_kind
    }

    pub fn min(&self) -> &ImmOrPNode<f64> {
        &self.min
    }

    pub fn max(&self) -> &ImmOrPNode<f64> {
        &self.max
    }

    pub fn inc(&self) -> Option<&ImmOrPNode<f64>> {
        self.inc.as_ref()
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn representation(&self) -> FloatRepresentation {
        self.representation
    }

    pub fn display_notation(&self) -> DisplayNotation {
        self.display_notation
    }

    pub fn display_precision(&self) -> i64 {
        self.display_precision
    }
}

impl Parse for FloatNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), FLOAT);

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while(P_INVALIDATOR);
        let streamable = node.parse_if(STREAMABLE).unwrap_or_default();
        let value_kind = node.parse();
        let min = node
            .parse_if(MIN)
            .or_else(|| node.parse_if(P_MIN))
            .unwrap_or(ImmOrPNode::Imm(f64::MIN));
        let max = node
            .parse_if(MAX)
            .or_else(|| node.parse_if(P_MAX))
            .unwrap_or(ImmOrPNode::Imm(f64::MAX));
        let inc = node.parse_if(INC).or_else(|| node.parse_if(P_INC));
        let unit = node.parse_if(UNIT);
        let representation = node.parse_if(REPRESENTATION).unwrap_or_default();
        let display_notation = node.parse_if(DISPLAY_NOTATION).unwrap_or_default();
        let display_precision = node.parse_if(DISPLAY_PRECISION).unwrap_or(6);

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
            display_notation,
            display_precision,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_float_node() {
        let xml = r#"
            <Float Name = "TestNode">
                <pInvalidator>Invalidator0</pInvalidator>
                <pInvalidator>Invalidator1</pInvalidator>
                <Streamable>Yes</Streamable>
                <Value>-.45E-6</Value>
                <Min>-INF</Min>
                <Max>INF</Max>
                <Inc>NaN</Inc>
                <Unit>dB</Unit>
                <Representation>Logarithmic</Representation>
                <DisplayNotation>Fixed</DisplayNotation>
                <DisplayPrecision>10</DisplayPrecision>
            </Float>
            "#;

        let node: FloatNode = xml::Document::from_str(xml).unwrap().root_node().parse();

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0], "Invalidator0");
        assert_eq!(p_invalidators[1], "Invalidator1");

        assert!(node.streamable());
        assert!(matches! {node.value_kind(), numeric_node_elem::ValueKind::Value(_)});
        assert_eq!(node.min(), &ImmOrPNode::Imm(f64::NEG_INFINITY));
        assert_eq!(node.max(), &ImmOrPNode::Imm(f64::INFINITY));
        assert!(node.inc().unwrap().imm().unwrap().is_nan());
        assert_eq!(node.unit(), Some("dB"));
        assert_eq!(node.representation(), FloatRepresentation::Logarithmic);
        assert_eq!(node.display_notation(), DisplayNotation::Fixed);
        assert_eq!(node.display_precision(), 10);
    }
}
