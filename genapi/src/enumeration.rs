use super::{
    elem_type::ImmOrPNode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{IntegerId, NodeId},
};

#[derive(Debug, Clone)]
pub struct EnumerationNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) entries: Vec<EnumEntryNode>,
    pub(crate) value: ImmOrPNode<IntegerId>,
    pub(crate) p_selected: Vec<NodeId>,
    pub(crate) polling_time: Option<u64>,
}

impl EnumerationNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn entries(&self) -> &[EnumEntryNode] {
        &self.entries
    }

    #[must_use]
    pub fn value(&self) -> ImmOrPNode<IntegerId> {
        self.value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

#[derive(Debug, Clone)]
pub struct EnumEntryNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) value: i64,
    pub(crate) numeric_values: Vec<f64>,
    pub(crate) symbolic: Option<String>,
    pub(crate) is_self_clearing: bool,
}

impl EnumEntryNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }

    #[must_use]
    pub fn numeric_values(&self) -> &[f64] {
        &self.numeric_values
    }

    #[must_use]
    pub fn symbolic(&self) -> Option<&str> {
        self.symbolic.as_deref()
    }

    pub fn set_symbolic(&mut self, s: String) {
        self.symbolic = Some(s)
    }

    #[must_use]
    pub fn is_self_clearing(&self) -> bool {
        self.is_self_clearing
    }
}
