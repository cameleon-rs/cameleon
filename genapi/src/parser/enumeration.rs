use crate::{node_store::NodeStore, EnumEntryNode, EnumerationNode};

use super::{
    elem_name::{
        ENUMERATION, ENUM_ENTRY, IS_SELF_CLEARING, NUMERIC_VALUE, POLLING_TIME, P_INVALIDATOR,
        P_SELECTED, STREAMABLE, SYMBOLIC,
    },
    xml, Parse,
};

impl Parse for EnumerationNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), ENUMERATION);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let mut entries = vec![];
        while let Some(mut ent_node) = node.next_if(ENUM_ENTRY) {
            entries.push(ent_node.parse(store));
        }
        let value = node.parse(store);
        let p_selected = node.parse_while(P_SELECTED, store);
        let polling_time = node.parse_if(POLLING_TIME, store);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            entries,
            value,
            p_selected,
            polling_time,
        }
    }
}

impl Parse for EnumEntryNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), ENUM_ENTRY);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let value = node.parse(store);
        let numeric_values = node.parse_while(NUMERIC_VALUE, store);
        let symbolic = node.parse_if(SYMBOLIC, store);
        let is_self_clearing = node.parse_if(IS_SELF_CLEARING, store).unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            p_invalidators,
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

        let mut store = NodeStore::new();
        let node: EnumerationNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.value(), &ImmOrPNode::PNode(store.id_by_name("MyNode")));
        assert_eq!(node.polling_time(), Some(10));

        let entries = node.entries();
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