use super::{
    elem_type::{numeric_node_elem, DisplayNotation, FloatRepresentation, ImmOrPNode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::NodeId,
};

#[derive(Debug, Clone)]
pub struct FloatNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_invalidators: Vec<NodeId>,
    pub(crate) streamable: bool,
    pub(crate) value_kind: numeric_node_elem::ValueKind<f64>,
    pub(crate) min: ImmOrPNode<f64>,
    pub(crate) max: ImmOrPNode<f64>,
    pub(crate) inc: Option<ImmOrPNode<f64>>,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
}

impl FloatNode {
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
    pub fn value_kind(&self) -> &numeric_node_elem::ValueKind<f64> {
        &self.value_kind
    }

    #[must_use]
    pub fn min(&self) -> &ImmOrPNode<f64> {
        &self.min
    }

    #[must_use]
    pub fn max(&self) -> &ImmOrPNode<f64> {
        &self.max
    }

    #[must_use]
    pub fn inc(&self) -> Option<&ImmOrPNode<f64>> {
        self.inc.as_ref()
    }

    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation(&self) -> FloatRepresentation {
        self.representation
    }

    #[must_use]
    pub fn display_notation(&self) -> DisplayNotation {
        self.display_notation
    }

    #[must_use]
    pub fn display_precision(&self) -> i64 {
        self.display_precision
    }
}
