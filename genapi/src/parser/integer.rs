use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    elem_type::{ImmOrPNode, IntegerRepresentation},
    store::NodeId,
    IntegerNode,
};

use super::{
    elem_name::{
        INC, INTEGER, MAX, MIN, P_INC, P_MAX, P_MIN, P_SELECTED, REPRESENTATION, STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for IntegerNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `IntegerNode`");
        debug_assert_eq!(node.tag_name(), INTEGER);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let value_kind = node.parse(node_builder, value_builder, cache_builder);
        let min = node
            .parse_if(MIN, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(P_MIN, node_builder, value_builder, cache_builder));
        let max = node
            .parse_if(MAX, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(P_MAX, node_builder, value_builder, cache_builder));
        let inc = node
            .parse_if(INC, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(P_INC, node_builder, value_builder, cache_builder))
            .unwrap_or(ImmOrPNode::Imm(10));
        let unit = node.parse_if(UNIT, node_builder, value_builder, cache_builder);
        let representation: IntegerRepresentation = node
            .parse_if(REPRESENTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let p_selected: Vec<NodeId> =
            node.parse_while(P_SELECTED, node_builder, value_builder, cache_builder);

        // Deduce min and max value based on representation if not specified.
        let min = min.unwrap_or_else(|| {
            let id = value_builder.store(representation.deduce_min());
            ImmOrPNode::Imm(id)
        });
        let max = max.unwrap_or_else(|| {
            let id = value_builder.store(representation.deduce_max());
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
    use crate::{elem_type::ValueKind, store::ValueStore};

    use super::{super::utils::tests::parse_default, *};

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

        let (node, mut node_builder, value_builder, _): (IntegerNode, _, _, _) = parse_default(xml);

        assert!(node.streamable());
        let value = value_builder
            .integer_value(node.value_kind.imm().unwrap())
            .unwrap();
        assert_eq!(value, 0x100);
        let min = value_builder
            .integer_value(node.min_elem().imm().unwrap())
            .unwrap();
        assert_eq!(min, 0x10);
        let max = value_builder
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
        assert_eq!(p_selected[0], node_builder.get_or_intern("Selected0"));
        assert_eq!(p_selected[1], node_builder.get_or_intern("Selected1"));
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

        let (node, mut node_builder, ..): (IntegerNode, _, _, _) = parse_default(xml);
        let p_value = match node.value_kind() {
            ValueKind::PValue(p_value) => p_value,
            _ => panic!(),
        };
        assert_eq!(p_value.p_value, node_builder.get_or_intern("pValue"));
        let p_value_copies = &p_value.p_value_copies;
        assert_eq!(p_value_copies.len(), 3);
        assert_eq!(p_value_copies[0], node_builder.get_or_intern("Copy1"));
        assert_eq!(p_value_copies[1], node_builder.get_or_intern("Copy2"));
        assert_eq!(p_value_copies[2], node_builder.get_or_intern("Copy3"));

        assert_eq!(
            node.min_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("pMinNode"))
        );
        assert_eq!(
            node.max_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("pMaxNode"))
        );
        assert_eq!(
            node.inc_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("pIncNode"))
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

        let (node, mut node_builder, value_builder, _): (IntegerNode, _, _, _) = parse_default(xml);
        let p_index = match node.value_kind {
            ValueKind::PIndex(p_index) => p_index,
            _ => panic!(),
        };

        assert_eq!(p_index.p_index, node_builder.get_or_intern("pIndexNode"));

        let value_indexed = p_index.value_indexed;
        assert_eq!(value_indexed.len(), 3);
        let value0 = value_builder
            .integer_value(value_indexed[0].indexed().imm().unwrap())
            .unwrap();
        assert_eq!(value0, 100);
        assert_eq!(value_indexed[0].index(), 10);

        assert_eq!(
            value_indexed[1].indexed,
            ImmOrPNode::PNode(node_builder.get_or_intern("pValueIndexNode"))
        );
        assert_eq!(value_indexed[1].index, 20);

        let value2 = value_builder
            .integer_value(value_indexed[2].indexed().imm().unwrap())
            .unwrap();
        assert_eq!(value2, 300);

        assert_eq!(
            p_index.value_default,
            ImmOrPNode::PNode(node_builder.get_or_intern("pValueDefaultNode"))
        );
    }
}
