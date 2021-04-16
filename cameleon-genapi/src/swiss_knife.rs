use super::{
    elem_type::{DisplayNotation, FloatRepresentation, NamedValue},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::NodeId,
};

#[derive(Debug, Clone)]
pub struct SwissKnifeNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_invalidators: Vec<NodeId>,
    pub(crate) streamable: bool,
    pub(crate) p_variables: Vec<NamedValue<NodeId>>,
    pub(crate) constants: Vec<NamedValue<f64>>,
    pub(crate) expressions: Vec<NamedValue<String>>,
    pub(crate) formula: String,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
}

impl SwissKnifeNode {
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
    pub fn p_variables(&self) -> &[NamedValue<NodeId>] {
        &self.p_variables
    }

    #[must_use]
    pub fn constants(&self) -> &[NamedValue<f64>] {
        &self.constants
    }

    #[must_use]
    pub fn expressions(&self) -> &[NamedValue<String>] {
        &self.expressions
    }

    #[must_use]
    pub fn formula(&self) -> &str {
        &self.formula
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
