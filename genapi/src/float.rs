use super::{
    elem_type::{DisplayNotation, FloatRepresentation, ImmOrPNode, ValueKind},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::FloatId,
};

#[derive(Debug, Clone)]
pub struct FloatNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) value_kind: ValueKind<FloatId>,
    pub(crate) min: ImmOrPNode<FloatId>,
    pub(crate) max: ImmOrPNode<FloatId>,
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
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn value_kind(&self) -> &ValueKind<FloatId> {
        &self.value_kind
    }

    #[must_use]
    pub fn min(&self) -> ImmOrPNode<FloatId> {
        self.min
    }

    #[must_use]
    pub fn max(&self) -> ImmOrPNode<FloatId> {
        self.max
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
