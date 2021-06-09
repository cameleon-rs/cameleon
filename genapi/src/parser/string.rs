/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    elem_type::ImmOrPNode,
    StringNode,
};

use super::{
    elem_name::{STREAMABLE, STRING, VALUE},
    xml, Parse,
};

impl Parse for StringNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `StringNode`");
        debug_assert_eq!(node.tag_name(), STRING);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let value = node.next_if(VALUE).map_or_else(
            || ImmOrPNode::PNode(node_builder.get_or_intern(&node.next_text().unwrap().view())),
            |next_node| {
                let id = value_builder.store(next_node.text().view().into_owned());
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
    use crate::store::ValueStore;

    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_string_with_imm() {
        let xml = r#"
        <String Name="TestNode">
            <Streamable>Yes</Streamable>
            <Value>Immediate String</Value>
        </String>
        "#;

        let (node, _, value_builder, _): (StringNode, _, _, _) = parse_default(xml);
        assert_eq!(node.streamable(), true);
        let value = value_builder
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

        let (node, mut node_builder, ..): (StringNode, _, _, _) = parse_default(xml);
        assert_eq!(node.streamable(), false);
        assert_eq!(
            node.value_elem(),
            ImmOrPNode::PNode(node_builder.get_or_intern("AnotherStringNode"))
        );
    }
}
