use crate::{
    elem_type::ImmOrPNode,
    store::{ValueStore, WritableNodeStore},
    StringNode,
};

use super::{
    elem_name::{STREAMABLE, STRING, VALUE},
    xml, Parse,
};

impl Parse for StringNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), STRING);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let streamable = node
            .parse_if(STREAMABLE, node_store, value_store)
            .unwrap_or_default();
        let value = node.next_if(VALUE).map_or_else(
            || ImmOrPNode::PNode(node_store.id_by_name(node.next_text().unwrap())),
            |next_node| {
                let id = value_store.store(String::from(next_node.text()));
                ImmOrPNode::Imm(id)
            },
        );

        Self {
            attr_base,
            elem_base,
            streamable,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::store::{DefaultNodeStore, DefaultValueStore};

    use super::*;

    #[test]
    fn test_string_with_imm() {
        let xml = r#"
        <String Name="TestNode">
            <Streamable>Yes</Streamable>
            <Value>Immediate String</Value>
        </String>
        "#;

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: StringNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        assert_eq!(node.streamable(), true);
        let value = value_store
            .str_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value, "Immediate String");
    }

    #[test]
    fn test_string_with_p_node() {
        let xml = r#"
        <String Name="TestNode">
            <pValue>AnotherStringNode</pValue>
        </String>
        "#;

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: StringNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        assert_eq!(node.streamable(), false);
        assert_eq!(
            node.value_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("AnotherStringNode"))
        );
    }
}
