use crate::node_store::NodeStore;

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
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), GROUP);
        let comment = node.attribute_of(COMMENT).unwrap().into();

        let mut nodes = vec![];
        while let Some(ref mut child) = node.next() {
            for data in child.parse::<Vec<NodeData>>(store) {
                nodes.push(data);
            }
        }

        Self { comment, nodes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let mut store = NodeStore::new();
        let node: GroupNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.comment, "Nothing to say");
        assert_eq!(node.nodes.len(), 2);
    }
}
