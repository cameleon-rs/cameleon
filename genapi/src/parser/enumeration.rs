use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    EnumEntryNode, EnumerationNode,
};

use super::{
    elem_name::{
        ENUMERATION, ENUM_ENTRY, IS_SELF_CLEARING, NUMERIC_VALUE, POLLING_TIME, P_SELECTED,
        STREAMABLE, SYMBOLIC,
    },
    xml, Parse,
};

impl Parse for EnumerationNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `EnumerationNode`");
        debug_assert_eq!(node.tag_name(), ENUMERATION);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let mut entries = vec![];
        while let Some(mut ent_node) = node.next_if(ENUM_ENTRY) {
            entries.push(ent_node.parse(node_builder, value_builder, cache_builder));
        }
        let value = node.parse(node_builder, value_builder, cache_builder);
        let p_selected = node.parse_while(P_SELECTED, node_builder, value_builder, cache_builder);
        let polling_time = node.parse_if(POLLING_TIME, node_builder, value_builder, cache_builder);

        Self {
            attr_base,
            elem_base,
            streamable,
            entries,
            value,
            p_selected,
            polling_time,
        }
    }
}

impl Parse for EnumEntryNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `EnumEntryNode`");
        debug_assert_eq!(node.tag_name(), ENUM_ENTRY);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let value = node.parse(node_builder, value_builder, cache_builder);
        let numeric_values = node.parse_while(NUMERIC_VALUE, node_builder, value_builder, cache_builder);
        let symbolic = node.parse_if(SYMBOLIC, node_builder, value_builder, cache_builder);
        let is_self_clearing = node
            .parse_if(IS_SELF_CLEARING, node_builder, value_builder, cache_builder)
            .unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            value,
            numeric_values,
            symbolic,
            is_self_clearing,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::elem_type::ImmOrPNode;

    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_enumeration() {
        let xml = r#"
            <Enumeration Name="TestNode">
                <EnumEntry Name="Entry0">
                    <Value>0</Value>
                    <NumericValue>1.0</NumericValue>
                    <NumericValue>10.0</NumericValue>
                    <IsSelfClearing>Yes</IsSelfClearing>
                </EnumEntry>
                <EnumEntry Name="Entry1">
                    <Value>1</Value>
                </EnumEntry>
                <pValue>MyNode</pValue>
            <PollingTime>10</PollingTime>
            </Enumeration>
            "#;

        let (node, mut node_builder, ..): (EnumerationNode, _, _, _) = parse_default(xml);

        assert_eq!(
            node.value_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("MyNode"))
        );
        assert_eq!(node.polling_time(), Some(10));

        let entries = node.entries_elem();
        assert_eq!(entries.len(), 2);

        let entry0 = &entries[0];
        assert_eq!(entry0.value(), 0);
        assert!((entry0.numeric_values()[0] - 1.0).abs() < f64::EPSILON);
        assert!((entry0.numeric_values()[1] - 10.0).abs() < f64::EPSILON);
        assert_eq!(entry0.is_self_clearing(), true);

        let entry1 = &entries[1];
        assert_eq!(entry1.value(), 1);
        assert_eq!(entry1.is_self_clearing(), false);
    }
}
