use tracing::debug;

use crate::{
    elem_type::{ImmOrPNode, IntegerRepresentation},
    store::{NodeId, ValueStore, WritableNodeStore},
    IntegerNode,
};

use super::{
    elem_name::{
        INC, INTEGER, MAX, MIN, P_INC, P_MAX, P_MIN, P_SELECTED, REPRESENTATION, STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for IntegerNode {
    #[tracing::instrument(level = "trace", skip(node_store, value_store))]
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug!("start parsing `IntegerNode`");
        debug_assert_eq!(node.tag_name(), INTEGER);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let streamable = node
            .parse_if(STREAMABLE, node_store, value_store)
            .unwrap_or_default();
        let value_kind = node.parse(node_store, value_store);
        let min = node
            .parse_if(MIN, node_store, value_store)
            .or_else(|| node.parse_if(P_MIN, node_store, value_store));
        let max = node
            .parse_if(MAX, node_store, value_store)
            .or_else(|| node.parse_if(P_MAX, node_store, value_store));
        let inc = node
            .parse_if(INC, node_store, value_store)
            .or_else(|| node.parse_if(P_INC, node_store, value_store))
            .unwrap_or(ImmOrPNode::Imm(10));
        let unit = node.parse_if(UNIT, node_store, value_store);
        let representation: IntegerRepresentation = node
            .parse_if(REPRESENTATION, node_store, value_store)
            .unwrap_or_default();
        let p_selected: Vec<NodeId> = node.parse_while(P_SELECTED, node_store, value_store);

        // Deduce min and max value based on representation if not specified.
        let min = min.unwrap_or_else(|| {
            let id = value_store.store(representation.deduce_min());
            ImmOrPNode::Imm(id)
        });
        let max = max.unwrap_or_else(|| {
            let id = value_store.store(representation.deduce_max());
            ImmOrPNode::Imm(id)
        });

        Self {
            attr_base,
            elem_base,
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
    use crate::{
        elem_type::ValueKind,
        store::{DefaultNodeStore, DefaultValueStore},
    };

    use super::*;

    fn integer_node_from_str(xml: &str) -> (IntegerNode, DefaultNodeStore, DefaultValueStore) {
        let document = xml::Document::from_str(xml).unwrap();
        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        (
            document
                .root_node()
                .parse(&mut node_store, &mut value_store),
            node_store,
            value_store,
        )
    }

    #[test]
    fn test_integer_node_with_immediate() {
        let xml = r#"
            <Integer Name = "TestNode">
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

        let (node, mut node_store, value_store) = integer_node_from_str(xml);

        assert!(node.streamable());
        let value = value_store
            .integer_value(node.value_kind.imm().unwrap())
            .unwrap();
        assert_eq!(value, 0x100);
        let min = value_store
            .integer_value(node.min_elem().imm().unwrap())
            .unwrap();
        assert_eq!(min, 0x10);
        let max = value_store
            .integer_value(node.max_elem().imm().unwrap())
            .unwrap();
        assert_eq!(max, 100);
        assert_eq!(node.inc_elem(), ImmOrPNode::Imm(0x5));
        assert_eq!(node.unit_elem(), Some("dB"));
        assert_eq!(
            node.representation_elem(),
            IntegerRepresentation::Logarithmic
        );

        let p_selected = node.p_selected();
        assert_eq!(p_selected.len(), 2);
        assert_eq!(p_selected[0], node_store.id_by_name("Selected0"));
        assert_eq!(p_selected[1], node_store.id_by_name("Selected1"));
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

        let (node, mut node_store, _) = integer_node_from_str(xml);
        let p_value = match node.value_kind() {
            ValueKind::PValue(p_value) => p_value,
            _ => panic!(),
        };
        assert_eq!(p_value.p_value, node_store.id_by_name("pValue"));
        let p_value_copies = &p_value.p_value_copies;
        assert_eq!(p_value_copies.len(), 3);
        assert_eq!(p_value_copies[0], node_store.id_by_name("Copy1"));
        assert_eq!(p_value_copies[1], node_store.id_by_name("Copy2"));
        assert_eq!(p_value_copies[2], node_store.id_by_name("Copy3"));

        assert_eq!(
            node.min_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("pMinNode"))
        );
        assert_eq!(
            node.max_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("pMaxNode"))
        );
        assert_eq!(
            node.inc_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("pIncNode"))
        );
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

        let (node, mut node_store, value_store) = integer_node_from_str(xml);
        let p_index = match node.value_kind {
            ValueKind::PIndex(p_index) => p_index,
            _ => panic!(),
        };

        assert_eq!(p_index.p_index, node_store.id_by_name("pIndexNode"));

        let value_indexed = p_index.value_indexed;
        assert_eq!(value_indexed.len(), 3);
        let value0 = value_store
            .integer_value(value_indexed[0].indexed().imm().unwrap())
            .unwrap();
        assert_eq!(value0, 100);
        assert_eq!(value_indexed[0].index(), 10);

        assert_eq!(
            value_indexed[1].indexed,
            ImmOrPNode::PNode(node_store.id_by_name("pValueIndexNode"))
        );
        assert_eq!(value_indexed[1].index, 20);

        let value2 = value_store
            .integer_value(value_indexed[2].indexed().imm().unwrap())
            .unwrap();
        assert_eq!(value2, 300);

        assert_eq!(
            p_index.value_default,
            ImmOrPNode::PNode(node_store.id_by_name("pValueDefaultNode"))
        );
    }
}
