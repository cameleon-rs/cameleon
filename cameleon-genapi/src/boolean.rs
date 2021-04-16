use super::{
    elem_type::ImmOrPNode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::NodeId,
};

#[derive(Debug, Clone)]
pub struct BooleanNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_invalidators: Vec<NodeId>,
    pub(crate) streamable: bool,
    pub(crate) value: ImmOrPNode<bool>,
    pub(crate) on_value: Option<i64>,
    pub(crate) off_value: Option<i64>,
    pub(crate) p_selected: Vec<NodeId>,
}

impl BooleanNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn value(&self) -> &ImmOrPNode<bool> {
        &self.value
    }

    #[must_use]
    pub fn on_value(&self) -> Option<i64> {
        self.on_value
    }

    #[must_use]
    pub fn off_value(&self) -> Option<i64> {
        self.off_value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}
