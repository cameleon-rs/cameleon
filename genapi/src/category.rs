use super::{
    interface::ICategory,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{NodeId, NodeStore},
    GenApiResult,
};

#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_features: Vec<NodeId>,
}

impl CategoryNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_features(&self) -> &[NodeId] {
        &self.p_features
    }
}

impl ICategory for CategoryNode {
    fn nodes(&self, _: impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_features())
    }
}
