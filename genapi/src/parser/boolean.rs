use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    elem_type::ImmOrPNode,
    BooleanNode,
};

use super::{
    elem_name::{BOOLEAN, OFF_VALUE, ON_VALUE, P_SELECTED, STREAMABLE},
    xml, Parse,
};

impl Parse for BooleanNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `BooleanNode`");
        debug_assert_eq!(node.tag_name(), BOOLEAN);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let value: ImmOrPNode<bool> = node.parse(node_builder, value_builder, cache_builder);
        let on_value: i64 = node
            .parse_if(ON_VALUE, node_builder, value_builder, cache_builder)
            .unwrap_or(1);
        let off_value: i64 = node
            .parse_if(OFF_VALUE, node_builder, value_builder, cache_builder)
            .unwrap_or(0);
        let p_selected = node.parse_while(P_SELECTED, node_builder, value_builder, cache_builder);

        let value = match value {
            ImmOrPNode::Imm(imm) => {
                let i = if imm { on_value } else { off_value };
                let id = value_builder.store(i);
                ImmOrPNode::Imm(id)
            }
            ImmOrPNode::PNode(pnode) => ImmOrPNode::PNode(pnode),
        };

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
    use crate::{elem_type::ImmOrPNode, store::ValueStore};

    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_boolean_node_with_p_node() {
        let xml = r#"
            <Boolean Name="TestNode">
                <pValue>N<!--comment here-->ode</pValue>
                <OnValue>10</OnValue>
                <OffValue>0</OffValue>
            </Boolean>
            "#;

        let (node, mut node_builder, ..): (BooleanNode, _, _, _) = parse_default(xml);
        assert_eq!(
            node.value_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("Node"))
        );
        assert_eq!(node.on_value(), 10);
        assert_eq!(node.off_value(), 0);
    }

    #[test]
    fn test_boolean_node_with_imm() {
        let xml1 = r#"
            <Boolean Name="TestNode">
                <Value>true</Value>
                <OnValue>10</OnValue>
                <OffValue>0</OffValue>
            </Boolean>
            "#;

        let (node, _, value_builder, ..): (BooleanNode, _, _, _) = parse_default(xml1);
        let value1 = value_builder
            .integer_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value1, node.on_value());

        let xml2 = r#"
            <Boolean Name="TestNode">
                <Value>false</Value>
                <OnValue>10</OnValue>
                <OffValue>0</OffValue>
            </Boolean>
            "#;

        let (node, _, value_builder2, ..): (BooleanNode, _, _, _) = parse_default(xml2);
        let value2 = value_builder2
            .integer_value(node.value_elem().imm().unwrap())
            .unwrap();
        assert_eq!(value2, node.off_value());
    }
}
