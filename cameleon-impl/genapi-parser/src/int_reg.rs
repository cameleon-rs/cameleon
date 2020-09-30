use super::{elem_type::*, node_base::*, register_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct IntRegNode {
    attr_base: NodeAttributeBase,

    register_base: RegisterBase,

    sign: register_node_elem::Sign,

    endianness: register_node_elem::Endianness,

    unit: Option<String>,

    representation: IntegerRepresentation,

    p_selected: Vec<String>,
}

impl IntRegNode {
    pub fn node_base(&self) -> NodeBase {
        let elem_base = self.register_base.elem_base();
        NodeBase::new(&self.attr_base, elem_base)
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    pub fn sign(&self) -> register_node_elem::Sign {
        self.sign
    }

    pub fn endianness(&self) -> register_node_elem::Endianness {
        self.endianness
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }
}

impl Parse for IntRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "IntReg");

        let attr_base = node.parse();
        let register_base = node.parse();

        let sign = node.parse_if("Sign").unwrap_or_default();

        let endianness = node.parse_if("Endianess").unwrap_or_default();

        let unit = node.parse_if("Unit");

        let representation = node.parse_if("Representation").unwrap_or_default();

        let mut p_selected = vec![];
        while let Some(selected) = node.parse_if("pSelected") {
            p_selected.push(selected);
        }

        Self {
            attr_base,
            register_base,
            sign,
            endianness,
            unit,
            representation,
            p_selected,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::register_node_elem::*;

    use super::*;

    #[test]
    fn test_int_reg() {
        let xml = r#"
        <IntReg Name="TestNode">
          <Address>0x10000</Address>
          <Length>4</Length>
          <pPort>Device</pPort>
          <Sign>Signed</Sign>
          <Endianess>BigEndian</Endianess>
          <Unit>Hz</Unit>
          <Representation>Logarithmic</Representation>
          <pSelected>SelectedNode</pSelected>
        </IntReg>
        "#;

        let node: IntRegNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.sign(), Sign::Signed);
        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.unit().unwrap(), "Hz");
        assert_eq!(node.representation(), IntegerRepresentation::Logarithmic);
        assert_eq!(node.p_selected().len(), 1);
    }
}
