use crate::{
    store::{ValueStore, WritableNodeStore},
    CommandNode,
};

use super::{
    elem_name::{COMMAND, POLLING_TIME},
    xml, Parse,
};

impl Parse for CommandNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), COMMAND);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let value = node.parse(node_store, value_store);
        let command_value = node.parse(node_store, value_store);
        let polling_time = node.parse_if(POLLING_TIME, node_store, value_store);

        Self {
            attr_base,
            elem_base,
            value,
            command_value,
            polling_time,
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
    fn test_command_node() {
        let xml = r#"
            <Command Name="TestNode">
                <Value>100</Value>
                <pCommandValue>CommandValueNode</pCommandValue>
                <PollingTime>1000</PollingTime>
            </Command>
            "#;

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();

        let node: CommandNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        let value = value_store
            .integer_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value, 100);
        assert_eq!(
            node.command_value_elem(),
            ImmOrPNode::PNode(node_store.id_by_name("CommandValueNode"))
        );
        assert_eq!(node.polling_time(), Some(1000));
    }
}
