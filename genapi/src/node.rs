use super::node_base::{NodeAttributeBase, NodeBase, NodeElementBase};

#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,
}

impl Node {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }
}
