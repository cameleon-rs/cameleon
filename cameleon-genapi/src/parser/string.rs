use crate::{elem_type::ImmOrPNode, node_store::NodeStore, StringNode};

use super::{
    elem_name::{P_INVALIDATOR, STREAMABLE, STRING, VALUE},
    xml, Parse,
};

impl Parse for StringNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), STRING);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let value = node.next_if(VALUE).map_or_else(
            || ImmOrPNode::PNode(store.id_by_name(node.next_text().unwrap())),
            |next_node| ImmOrPNode::Imm(next_node.text().into()),
        );

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_with_imm() {
        let xml = r#"
        <String Name="TestNode">
            <Streamable>Yes</Streamable>
            <Value>Immediate String</Value>
        </String>
        "#;

        let mut store = NodeStore::new();
        let node: StringNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.streamable(), true);
        assert_eq!(node.value(), &ImmOrPNode::Imm("Immediate String".into()));
    }

    #[test]
    fn test_string_with_p_node() {
        let xml = r#"
        <String Name="TestNode">
            <pValue>AnotherStringNode</pValue>
        </String>
        "#;

        let mut store = NodeStore::new();
        let node: StringNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.streamable(), false);
        assert_eq!(
            node.value(),
            &ImmOrPNode::PNode(store.id_by_name("AnotherStringNode"))
        );
    }
}
