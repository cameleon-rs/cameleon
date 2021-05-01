use tracing::debug;

use crate::{
    store::{ValueStore, WritableNodeStore},
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
    #[tracing::instrument(level = "trace", skip(node_store, value_store))]
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug!("start parsing `EnumerationNode`");
        debug_assert_eq!(node.tag_name(), ENUMERATION);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let streamable = node
            .parse_if(STREAMABLE, node_store, value_store)
            .unwrap_or_default();
        let mut entries = vec![];
        while let Some(mut ent_node) = node.next_if(ENUM_ENTRY) {
            entries.push(ent_node.parse(node_store, value_store));
        }
        let value = node.parse(node_store, value_store);
        let p_selected = node.parse_while(P_SELECTED, node_store, value_store);
        let polling_time = node.parse_if(POLLING_TIME, node_store, value_store);

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
    #[tracing::instrument(level = "trace", skip(node_store, value_store))]
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug!("start parsing `EnumEntryNode`");
        debug_assert_eq!(node.tag_name(), ENUM_ENTRY);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let value = node.parse(node_store, value_store);
        let numeric_values = node.parse_while(NUMERIC_VALUE, node_store, value_store);
        let symbolic = node.parse_if(SYMBOLIC, node_store, value_store);
        let is_self_clearing = node
            .parse_if(IS_SELF_CLEARING, node_store, value_store)
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
    use crate::{
        elem_type::ImmOrPNode,
        store::{DefaultNodeStore, DefaultValueStore},
    };

    use super::*;

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

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: EnumerationNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        assert_eq!(
            node.value_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("MyNode"))
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
