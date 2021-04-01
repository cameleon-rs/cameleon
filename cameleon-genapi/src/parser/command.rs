use super::{
    elem_name::{COMMAND, POLLING_TIME, P_INVALIDATOR},
    elem_type::ImmOrPNode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct CommandNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    value: ImmOrPNode<i64>,
    command_value: ImmOrPNode<i64>,
    polling_time: Option<u64>,
}

impl CommandNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn value(&self) -> &ImmOrPNode<i64> {
        &self.value
    }

    #[must_use]
    pub fn command_value(&self) -> &ImmOrPNode<i64> {
        &self.command_value
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

impl Parse for CommandNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
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

        let mut store = NodeStore::new();
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
