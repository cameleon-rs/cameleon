use crate::{store::NodeStore, BooleanNode};

use super::{
    elem_name::{BOOLEAN, OFF_VALUE, ON_VALUE, P_INVALIDATOR, P_SELECTED, STREAMABLE},
    xml, Parse,
};

impl Parse for BooleanNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), BOOLEAN);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let value = node.parse(store);
        let on_value = node.parse_if(ON_VALUE, store);
        let off_value = node.parse_if(OFF_VALUE, store);
        let p_selected = node.parse_while(P_SELECTED, store);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
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
    use crate::{elem_type::ImmOrPNode, store::DefaultNodeStore};

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

        let mut store = DefaultNodeStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);
        assert_eq!(node.value(), &ImmOrPNode::PNode(store.id_by_name("Node")));
        assert_eq!(node.on_value(), Some(1));
        assert_eq!(node.off_value(), Some(0));
    }

    #[test]
    fn test_boolean_node_with_imm() {
        let xml1 = r#"
            <Boolean Name="TestNode">
                <Value>true</Value>
            </Boolean>
            "#;

        let mut store = DefaultNodeStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml1)
            .unwrap()
            .root_node()
            .parse(&mut store);
        assert_eq!(node.value(), &ImmOrPNode::Imm(true));

        let xml2 = r#"
            <Boolean Name="TestNode">
                <Value>false</Value>
            </Boolean>
            "#;

        let mut store2 = DefaultNodeStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml2)
            .unwrap()
            .root_node()
            .parse(&mut store2);
        assert_eq!(node.value(), &ImmOrPNode::Imm(false));
    }
}
