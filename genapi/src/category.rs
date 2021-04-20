use super::{
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::NodeId,
};

#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_invalidators: Vec<NodeId>,
    pub(crate) p_features: Vec<NodeId>,
}

impl CategoryNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn p_features(&self) -> &[NodeId] {
        &self.p_features
    }
}
