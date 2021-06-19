/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    node_base::NodeAttributeBase,
    store::NodeData,
    EnumEntryNode, EnumerationNode,
};

use super::{
    elem_name::{
        ENUMERATION, ENUM_ENTRY, EXPOSE_STATIC, IS_SELF_CLEARING, MERGE_PRIORITY, NAME, NAME_SPACE,
        NUMERIC_VALUE, POLLING_TIME, P_SELECTED, STREAMABLE,
    },
    elem_type::convert_to_bool,
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
            let entry: EnumEntryNode = ent_node.parse(node_builder, value_builder, cache_builder);
            let nid = entry.attr_base.id;
            node_builder.store_node(nid, NodeData::EnumEntry(entry.into()));
            entries.push(nid);
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

        // We can't use `NodeAttributeBase::parse` for needs of generating fresh symbol.
        let symbolic = node.attribute_of(NAME).unwrap().to_string();
        let name = format!("${}_{}", symbolic, node_builder.fresh_id());
        let id = node_builder.get_or_intern(&name);
        let name_space = node
            .attribute_of(NAME_SPACE)
            .map(|text| text.into())
            .unwrap_or_default();
        let merge_priority = node
            .attribute_of(MERGE_PRIORITY)
            .map(|text| text.into())
            .unwrap_or_default();
        let expose_static = node
            .attribute_of(EXPOSE_STATIC)
            .map(|text| convert_to_bool(text));

        let attr_base = NodeAttributeBase {
            id,
            name_space,
            merge_priority,
            expose_static,
        };
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let value = node.parse(node_builder, value_builder, cache_builder);
        let numeric_value =
            node.parse_if(NUMERIC_VALUE, node_builder, value_builder, cache_builder);
        let is_self_clearing = node
            .parse_if(IS_SELF_CLEARING, node_builder, value_builder, cache_builder)
            .unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            value,
            numeric_value,
            symbolic,
            is_self_clearing,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{elem_type::ImmOrPNode, interface::IEnumeration};

    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_enumeration() {
        let xml = r#"
            <Enumeration Name="TestNode">
                <EnumEntry Name="Entry0">
                    <Value>0</Value>
                    <NumericValue>1.0</NumericValue>
                    <IsSelfClearing>Yes</IsSelfClearing>
                </EnumEntry>
                <EnumEntry Name="Entry1">
                    <Value>1</Value>
                    <NumericValue>10.0</NumericValue>
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

        let entries = node.entries(&node_builder);
        assert_eq!(entries.len(), 2);

        let entry0 = &entries[0].expect_enum_entry(&node_builder).unwrap();
        assert_eq!(entry0.symbolic(), "Entry0");
        assert_eq!(entry0.value(), 0);
        assert!((entry0.numeric_value() - 1.0).abs() < f64::EPSILON);
        assert!(entry0.is_self_clearing());

        let entry1 = &entries[1].expect_enum_entry(&node_builder).unwrap();
        assert_eq!(entry1.symbolic(), "Entry1");
        assert_eq!(entry1.value(), 1);
        assert!((entry1.numeric_value() - 10.0).abs() < f64::EPSILON);
        assert!(!entry1.is_self_clearing());
    }
}
