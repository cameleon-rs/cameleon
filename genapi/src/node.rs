use super::{
    interface::INode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
};

#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,
}

impl INode for Node {
    fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        false
    }
}
