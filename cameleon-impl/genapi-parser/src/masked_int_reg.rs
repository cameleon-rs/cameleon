use super::{
    elem_name::{ENDIANNESS, MASKED_INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    elem_type::{register_node_elem, IntegerRepresentation},
    node_base::{NodeAttributeBase, NodeBase},
    register_base::RegisterBase,
    xml, Parse,
};

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
    pub fn bit_mask(&self) -> register_node_elem::BitMask {
        self.bit_mask
    }

    #[must_use]
    pub fn sign(&self) -> register_node_elem::Sign {
        self.sign
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
    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }
}

impl Parse for MaskedIntRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), MASKED_INT_REG);
        let attr_base = node.parse();

        let register_base = node.parse();
        let bit_mask = node.parse();
        let sign = node.parse_if(SIGN).unwrap_or_default();
        let endianness = node.parse_if(ENDIANNESS).unwrap_or_default();
        let unit = node.parse_if(UNIT);
        let representation = node.parse_if(REPRESENTATION).unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED);

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
    use crate::register_node_elem::BitMask;

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
