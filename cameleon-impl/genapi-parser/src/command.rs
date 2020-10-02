use super::{elem_type::*, node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct CommandNode {
    attr_base: NodeAttributeBase,

    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,

    value: ImmOrPNode<i64>,

    command_value: ImmOrPNode<i64>,

    polling_time: Option<u64>,
}

impl CommandNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn value(&self) -> &ImmOrPNode<i64> {
        &self.value
    }

    pub fn command_value(&self) -> &ImmOrPNode<i64> {
        &self.command_value
    }

    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

impl Parse for CommandNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), "Command");
        let attr_base = node.parse();
        let elem_base = node.parse();

        let mut p_invalidators = vec![];
        for invalidator in node.parse_if("pInvalidator") {
            p_invalidators.push(invalidator);
        }

        let value = node.parse();

        let command_value = node.parse();

        let polling_time = node.parse_if("PollingTime");

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

        let node: CommandNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.value(), &ImmOrPNode::Imm(100));
        assert_eq!(
            node.command_value(),
            &ImmOrPNode::PNode("CommandValueNode".into())
        );
        assert_eq!(node.polling_time(), Some(1000));
    }
}
