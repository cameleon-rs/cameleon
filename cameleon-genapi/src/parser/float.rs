use crate::{elem_type::ImmOrPNode, node_store::NodeStore, FloatNode};

use super::{
    elem_name::{
        DISPLAY_NOTATION, DISPLAY_PRECISION, FLOAT, INC, MAX, MIN, P_INC, P_INVALIDATOR, P_MAX,
        P_MIN, REPRESENTATION, STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for FloatNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), FLOAT);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let value_kind = node.parse(store);
        let min = node
            .parse_if(MIN, store)
            .or_else(|| node.parse_if(P_MIN, store))
            .unwrap_or(ImmOrPNode::Imm(f64::MIN));
        let max = node
            .parse_if(MAX, store)
            .or_else(|| node.parse_if(P_MAX, store))
            .unwrap_or(ImmOrPNode::Imm(f64::MAX));
        let inc = node
            .parse_if(INC, store)
            .or_else(|| node.parse_if(P_INC, store));
        let unit = node.parse_if(UNIT, store);
        let representation = node.parse_if(REPRESENTATION, store).unwrap_or_default();
        let display_notation = node.parse_if(DISPLAY_NOTATION, store).unwrap_or_default();
        let display_precision = node.parse_if(DISPLAY_PRECISION, store).unwrap_or(6);

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
    use crate::elem_type::{numeric_node_elem, DisplayNotation, FloatRepresentation, ImmOrPNode};

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

        let mut store = NodeStore::new();
        let node: FloatNode = xml::Document::from_str(xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0], store.id_by_name("Invalidator0"));
        assert_eq!(p_invalidators[1], store.id_by_name("Invalidator1"));

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
