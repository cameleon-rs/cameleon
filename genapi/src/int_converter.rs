use super::{
    elem_type::{IntegerRepresentation, NamedValue, Slope},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::NodeId,
};

#[derive(Debug, Clone)]
pub struct IntConverterNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) p_variables: Vec<NamedValue<NodeId>>,
    pub(crate) constants: Vec<NamedValue<i64>>,
    pub(crate) expressions: Vec<NamedValue<String>>,
    pub(crate) formula_to: String,
    pub(crate) formula_from: String,
    pub(crate) p_value: NodeId,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
    pub(crate) slope: Slope,
}

impl IntConverterNode {
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
    pub fn formula_to(&self) -> &str {
        &self.formula_to
    }

    #[must_use]
    pub fn formula_from(&self) -> &str {
        &self.formula_from
    }

    #[must_use]
    pub fn p_value(&self) -> NodeId {
        self.p_value
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
    pub fn slope(&self) -> Slope {
        self.slope
    }
}
