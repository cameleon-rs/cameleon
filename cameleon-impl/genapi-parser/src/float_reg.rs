use super::{elem_type::*, node_base::*, register_base::*, xml, Parse};

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
    pub fn node_base(&self) -> NodeBase {
        let elem_base = self.register_base.elem_base();
        NodeBase::new(&self.attr_base, elem_base)
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    pub fn endianness(&self) -> register_node_elem::Endianness {
        self.endianness
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn representation(&self) -> FloatRepresentation {
        self.representation
    }

    pub fn display_notation(&self) -> DisplayNotation {
        self.display_notation
    }

    pub fn display_precision(&self) -> i64 {
        self.display_precision
    }
}

impl Parse for FloatRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "FloatReg");

        let attr_base = node.parse();
        let register_base = node.parse();

        let endianness = node.parse_if("Endianess").unwrap_or_default();
        let unit = node.parse_if("Unit");
        let representation = node.parse_if("Representation").unwrap_or_default();
        let display_notation = node.parse_if("DisplayNotation").unwrap_or_default();
        let display_precision = node.parse_if("DisplayPrecision").unwrap_or(6);

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
