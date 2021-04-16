use super::{
    elem_type::{register_node_elem, DisplayNotation, FloatRepresentation},
    node_base::{NodeAttributeBase, NodeBase},
    RegisterBase,
};

#[derive(Debug, Clone)]
pub struct FloatRegNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) register_base: RegisterBase,

    pub(crate) endianness: register_node_elem::Endianness,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
}

impl FloatRegNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    #[must_use]
    pub fn endianness(&self) -> register_node_elem::Endianness {
        self.endianness
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
