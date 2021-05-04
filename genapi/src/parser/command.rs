use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    CommandNode,
};

use super::{
    elem_name::{COMMAND, POLLING_TIME},
    xml, Parse,
};

impl Parse for CommandNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `CommandNode`");
        debug_assert_eq!(node.tag_name(), COMMAND);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let value = node.parse(node_builder, value_builder, cache_builder);
        let command_value = node.parse(node_builder, value_builder, cache_builder);
        let polling_time = node.parse_if(POLLING_TIME, node_builder, value_builder, cache_builder);

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
    use crate::{elem_type::ImmOrPNode, store::ValueStore};

    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_command_node() {
        let xml = r#"
            <Command Name="TestNode">
                <Value>100</Value>
                <pCommandValue>CommandValueNode</pCommandValue>
                <PollingTime>1000</PollingTime>
            </Command>
            "#;

        let (node, mut node_builder, value_builder, _): (CommandNode, _, _, _) = parse_default(xml);
        let value = value_builder
            .integer_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value, 100);
        assert_eq!(
            node.command_value_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("CommandValueNode"))
        );
        assert_eq!(node.polling_time(), Some(1000));
    }
}
