use super::{elem_type::*, node_base::*, register_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct MaskedIntRegNode {
    pub(super) attr_base: NodeAttributeBase,

    pub(super) register_base: RegisterBase,

    pub(super) bit_mask: register_node_elem::BitMask,

    pub(super) sign: register_node_elem::Sign,

    pub(super) endianness: register_node_elem::Endianness,

    pub(super) unit: Option<String>,

    pub(super) representation: IntegerRepresentation,

    pub(super) p_selected: Vec<String>,
}

impl MaskedIntRegNode {
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    pub fn bit_mask(&self) -> register_node_elem::BitMask {
        self.bit_mask
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

impl Parse for MaskedIntRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "MaskedIntReg");
        let attr_base = node.parse();

        let register_base = node.parse();

        let bit_mask = node.parse();

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
            bit_mask,
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
    fn test_masked_int_reg_with_single_bit_mask() {
        let xml = r#"
        <MaskedIntReg Name="TestNode">
          <Address>0x10000</Address>
          <Length>4</Length>
          <pPort>Device</pPort>
          <Bit>3</Bit>
        </MaskedIntReg>
        "#;

        let node: MaskedIntRegNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        debug_assert_eq!(node.bit_mask(), BitMask::SingleBit(3));
    }

    #[test]
    fn test_masked_int_reg_with_bit_range() {
        let xml = r#"
        <MaskedIntReg Name="TestNode">
          <Address>0x10000</Address>
          <Length>4</Length>
          <pPort>Device</pPort>
          <LSB>3</LSB>
          <MSB>7</MSB>
        </MaskedIntReg>
        "#;

        let node: MaskedIntRegNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        debug_assert_eq!(node.bit_mask(), BitMask::Range { lsb: 3, msb: 7 });
    }
}
