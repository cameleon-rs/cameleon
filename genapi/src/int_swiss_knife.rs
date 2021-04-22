use super::{
    elem_type::{IntegerRepresentation, NamedValue},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::NodeId,
};

#[derive(Debug, Clone)]
pub struct IntSwissKnifeNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) p_variables: Vec<NamedValue<NodeId>>,
    pub(crate) constants: Vec<NamedValue<i64>>,
    pub(crate) expressions: Vec<NamedValue<String>>,
    pub(crate) formula: String,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
}

impl IntSwissKnifeNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
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
    pub fn constants(&self) -> &[NamedValue<i64>] {
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
    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }
}
