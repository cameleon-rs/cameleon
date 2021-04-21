use crate::{store::NodeStore, CommandNode};

use super::{
    elem_name::{COMMAND, POLLING_TIME, P_INVALIDATOR},
    xml, Parse,
};

impl Parse for CommandNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), COMMAND);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let value = node.parse(store);
        let command_value = node.parse(store);
        let polling_time = node.parse_if(POLLING_TIME, store);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            value,
            command_value,
            polling_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{elem_type::ImmOrPNode, store::DefaultNodeStore};

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

        let mut store = DefaultNodeStore::new();
        let node: CommandNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.value(), &ImmOrPNode::Imm(100));
        assert_eq!(
            node.command_value(),
            &ImmOrPNode::PNode(store.id_by_name("CommandValueNode"))
        );
        assert_eq!(node.polling_time(), Some(1000));
    }
}
