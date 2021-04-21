use super::{
    elem_type::ImmOrPNode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{IntegerId, NodeId},
};

#[derive(Debug, Clone)]
pub struct CommandNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_invalidators: Vec<NodeId>,
    pub(crate) value: ImmOrPNode<IntegerId>,
    pub(crate) command_value: ImmOrPNode<IntegerId>,
    pub(crate) polling_time: Option<u64>,
}

impl CommandNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn value(&self) -> ImmOrPNode<IntegerId> {
        self.value
    }

    #[must_use]
    pub fn command_value(&self) -> ImmOrPNode<IntegerId> {
        self.command_value
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}
