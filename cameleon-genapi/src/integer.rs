use super::{
    elem_type::{numeric_node_elem, ImmOrPNode, IntegerRepresentation},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::NodeId,
};

#[derive(Debug, Clone)]
pub struct IntegerNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_invalidators: Vec<NodeId>,
    pub(crate) streamable: bool,
    pub(crate) value_kind: numeric_node_elem::ValueKind<i64>,
    pub(crate) min: ImmOrPNode<i64>,
    pub(crate) max: ImmOrPNode<i64>,
    pub(crate) inc: ImmOrPNode<i64>,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
    pub(crate) p_selected: Vec<NodeId>,
}

impl IntegerNode {
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
    pub fn value_kind(&self) -> &numeric_node_elem::ValueKind<i64> {
        &self.value_kind
    }

    #[must_use]
    pub fn min(&self) -> &ImmOrPNode<i64> {
        &self.min
    }

    #[must_use]
    pub fn max(&self) -> &ImmOrPNode<i64> {
        &self.max
    }

    #[must_use]
    pub fn inc(&self) -> &ImmOrPNode<i64> {
        &self.inc
    }

    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}
