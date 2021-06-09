/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use tracing::debug;

use crate::builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder};

use super::{
    elem_name::{COMMENT, GROUP},
    xml, NodeData, Parse,
};

#[derive(Debug, Clone)]
pub(super) struct GroupNode {
    comment: String,

    pub(super) nodes: Vec<NodeData>,
}

impl Parse for GroupNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `GroupNode`");
        debug_assert_eq!(node.tag_name(), GROUP);
        let comment = node.attribute_of(COMMENT).unwrap().into();

        let mut nodes = vec![];
        while let Some(ref mut child) = node.next() {
            let children: Vec<NodeData> = child.parse(node_builder, value_builder, cache_builder);
            for data in children {
                nodes.push(data);
            }
        }

        Self { comment, nodes }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_group_node() {
        let xml = r#"
            <Group Comment="Nothing to say">
                <IntReg Name="MyIntReg">
                  <Address>0x10000</Address>
                  <pLength>LengthNode</pLength>
                  <pPort>Device</pPort>
                </IntReg>
                <Port Name="MyPort">
                    <ChunkID>Fd3219</ChunkID>
                    <SwapEndianess>Yes</SwapEndianess>
                </Port>
            </Group>
            "#;

        let (node, ..): (GroupNode, _, _, _) = parse_default(xml);

        assert_eq!(node.comment, "Nothing to say");
        assert_eq!(node.nodes.len(), 2);
    }
}
