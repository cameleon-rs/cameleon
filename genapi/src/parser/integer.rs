use crate::{
    elem_type::{ImmOrPNode, IntegerRepresentation},
    node_store::{NodeId, NodeStore},
    IntegerNode,
};

use super::{
    elem_name::{
        INC, INTEGER, MAX, MIN, P_INC, P_INVALIDATOR, P_MAX, P_MIN, P_SELECTED, REPRESENTATION,
        STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for IntegerNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), INTEGER);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let value_kind = node.parse(store);
        let min = node
            .parse_if(MIN, store)
            .or_else(|| node.parse_if(P_MIN, store));
        let max = node
            .parse_if(MAX, store)
            .or_else(|| node.parse_if(P_MAX, store));
        let inc = node
            .parse_if(INC, store)
            .or_else(|| node.parse_if(P_INC, store))
            .unwrap_or(ImmOrPNode::Imm(10));
        let unit = node.parse_if(UNIT, store);
        let representation = node
            .parse_if::<IntegerRepresentation>(REPRESENTATION, store)
            .unwrap_or_default();
        let p_selected: Vec<NodeId> = node.parse_while(P_SELECTED, store);

        // Deduce min and max value based on representation if not specified.
        let min = min.unwrap_or_else(|| ImmOrPNode::Imm(representation.deduce_min()));
        let max = max.unwrap_or_else(|| ImmOrPNode::Imm(representation.deduce_max()));

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

#[cfg(test)]
mod tests {
    use crate::elem_type::ValueKind;

    use super::*;

    fn integer_node_from_str(xml: &str) -> (IntegerNode, NodeStore) {
        let document = xml::Document::from_str(xml).unwrap();
        let mut store = NodeStore::new();
        (document.root_node().parse(&mut store), store)
    }

    #[test]
    fn test_integer_node_with_immediate() {
        let xml = r#"
            <Integer Name = "TestNode">
                <pInvalidator>Invalidator0</pInvalidator>
                <pInvalidator>Invalidator1</pInvalidator>
                <Streamable>Yes</Streamable>
                <Value>0X100</Value>
                <Min>0x10</Min>
                <Max>100</Max>
                <Inc>0x5</Inc>
                <Unit>dB</Unit>
                <Representation>Logarithmic</Representation>
                <pSelected>Selected0</pSelected>
                <pSelected>Selected1</pSelected>

            </Integer>
            "#;

        let (node, mut store) = integer_node_from_str(xml);

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0], store.id_by_name("Invalidator0"));
        assert_eq!(p_invalidators[1], store.id_by_name("Invalidator1"));

        assert!(node.streamable());
        assert!(matches! {node.value_kind(), ValueKind::Value(0x100)});
        assert_eq!(node.min(), &ImmOrPNode::Imm(0x10));
        assert_eq!(node.max(), &ImmOrPNode::Imm(100));
        assert_eq!(node.inc(), &ImmOrPNode::Imm(0x5));
        assert_eq!(node.unit(), Some("dB"));
        assert_eq!(node.representation(), IntegerRepresentation::Logarithmic);

        let p_selected = node.p_selected();
        assert_eq!(p_selected.len(), 2);
        assert_eq!(p_selected[0], store.id_by_name("Selected0"));
        assert_eq!(p_selected[1], store.id_by_name("Selected1"));
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

        let (node, mut store) = integer_node_from_str(xml);
        let p_value = match node.value_kind() {
            ValueKind::PValue(p_value) => p_value,
            _ => panic!(),
        };
        assert_eq!(p_value.p_value, store.id_by_name("pValue"));
        let p_value_copies = &p_value.p_value_copies;
        assert_eq!(p_value_copies.len(), 3);
        assert_eq!(p_value_copies[0], store.id_by_name("Copy1"));
        assert_eq!(p_value_copies[1], store.id_by_name("Copy2"));
        assert_eq!(p_value_copies[2], store.id_by_name("Copy3"));

        assert_eq!(node.min(), &ImmOrPNode::PNode(store.id_by_name("pMinNode")));
        assert_eq!(node.max(), &ImmOrPNode::PNode(store.id_by_name("pMaxNode")));
        assert_eq!(node.inc(), &ImmOrPNode::PNode(store.id_by_name("pIncNode")));
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

        let (node, mut store) = integer_node_from_str(xml);
        let p_index = match node.value_kind {
            ValueKind::PIndex(p_index) => p_index,
            _ => panic!(),
        };

        assert_eq!(p_index.p_index, store.id_by_name("pIndexNode"));

        let value_indexed = p_index.value_indexed;
        assert_eq!(value_indexed.len(), 3);
        assert!(matches! {value_indexed[0].indexed, ImmOrPNode::Imm(100)});
        assert_eq!(value_indexed[0].index, 10);

        assert_eq!(
            value_indexed[1].indexed,
            ImmOrPNode::PNode(store.id_by_name("pValueIndexNode"))
        );
        assert_eq!(value_indexed[1].index, 20);

        assert!(matches! {value_indexed[2].indexed, ImmOrPNode::Imm(300)});
        assert_eq!(value_indexed[2].index, 30);

        assert_eq!(
            p_index.value_default,
            ImmOrPNode::PNode(store.id_by_name("pValueDefaultNode"))
        );
    }
}
