use super::{elem_name::*, xml, NodeKind, Parse};

#[derive(Debug, Clone)]
pub struct GroupNode {
    comment: String,

    nodes: Vec<NodeKind>,
}

impl GroupNode {
    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn nodes(&self) -> &[NodeKind] {
        &self.nodes
    }

    pub fn into_nodes(self) -> Vec<NodeKind> {
        self.nodes
    }
}

impl Parse for GroupNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), GROUP);
        let comment = node.attribute_of(COMMENT).unwrap().into();

        let mut nodes = vec![];
        while let Some(ref mut child) = node.next() {
            nodes.push(child.parse());
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

        let node: GroupNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.comment(), "Nothing to say");
        assert_eq!(node.nodes().len(), 2);
    }
}
