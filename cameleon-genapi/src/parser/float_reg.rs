use super::{
    elem_name::{DISPLAY_NOTATION, DISPLAY_PRECISION, ENDIANNESS, FLOAT_REG, REPRESENTATION, UNIT},
    elem_type::{register_node_elem, DisplayNotation, FloatRepresentation},
    node_base::{NodeAttributeBase, NodeBase},
    register_base::RegisterBase,
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct FloatRegNode {
    attr_base: NodeAttributeBase,
    register_base: RegisterBase,

    endianness: register_node_elem::Endianness,
    unit: Option<String>,
    representation: FloatRepresentation,
    display_notation: DisplayNotation,
    display_precision: i64,
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

impl Parse for FloatRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), FLOAT_REG);

        let attr_base = node.parse();
        let register_base = node.parse();

        let endianness = node.parse_if(ENDIANNESS).unwrap_or_default();
        let unit = node.parse_if(UNIT);
        let representation = node.parse_if(REPRESENTATION).unwrap_or_default();
        let display_notation = node.parse_if(DISPLAY_NOTATION).unwrap_or_default();
        let display_precision = node.parse_if(DISPLAY_PRECISION).unwrap_or(6);

        Self {
            attr_base,
            register_base,
            endianness,
            unit,
            representation,
            display_notation,
            display_precision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_reg() {
        let xml = r#"
        <FloatReg Name="TestNode">
          <Address>0x10000</Address>
          <Length>4</Length>
          <pPort>Device</pPort>
          <Endianess>BigEndian</Endianess>
          <Unit>Hz</Unit>
          <Representation>Linear</Representation>
          <DisplayNotation>Fixed</DisplayNotation>
          <DisplayPrecision>10</DisplayPrecision>
        </FloatReg>
        "#;

        let node: FloatRegNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.endianness(), register_node_elem::Endianness::BE);
        assert_eq!(node.unit().unwrap(), "Hz");
        assert_eq!(node.representation(), FloatRepresentation::Linear);
        assert_eq!(node.display_notation(), DisplayNotation::Fixed);
        assert_eq!(node.display_precision(), 10);
    }
}
