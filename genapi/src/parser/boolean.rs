use crate::{
    store::{WritableNodeStore, ValueStore},
    BooleanNode,
};

use super::{
    elem_name::{BOOLEAN, OFF_VALUE, ON_VALUE, P_SELECTED, STREAMABLE},
    xml, Parse,
};

impl Parse for BooleanNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), BOOLEAN);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let streamable = node
            .parse_if(STREAMABLE, node_store, value_store)
            .unwrap_or_default();
        let value = node.parse(node_store, value_store);
        let on_value = node
            .parse_if(ON_VALUE, node_store, value_store)
            .unwrap_or(1);
        let off_value = node
            .parse_if(OFF_VALUE, node_store, value_store)
            .unwrap_or(0);
        let p_selected = node.parse_while(P_SELECTED, node_store, value_store);

        Self {
            attr_base,
            elem_base,
            streamable,
            value,
            on_value,
            off_value,
            p_selected,
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
    fn test_boolean_node_with_p_node() {
        let xml = r#"
            <Boolean Name="TestNode">
                <pValue>Node</pValue>
                <OnValue>1</OnValue>
                <OffValue>0</OffValue>
            </Boolean>
            "#;

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);
        assert_eq!(
            node.value_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("Node"))
        );
        assert_eq!(node.on_value(), 1);
        assert_eq!(node.off_value(), 0);
    }

    #[test]
    fn test_boolean_node_with_imm() {
        let xml1 = r#"
            <Boolean Name="TestNode">
                <Value>true</Value>
            </Boolean>
            "#;

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml1)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);
        let value = value_store
            .boolean_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value, true);

        let xml2 = r#"
            <Boolean Name="TestNode">
                <Value>false</Value>
            </Boolean>
            "#;

        let mut node_store2 = DefaultNodeStore::new();
        let mut value_store2 = DefaultValueStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml2)
            .unwrap()
            .root_node()
            .parse(&mut node_store2, &mut value_store2);
        let value = value_store2
            .boolean_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value, false);
    }
}
