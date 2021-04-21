use crate::{store::NodeStore, MaskedIntRegNode};

use super::{
    elem_name::{ENDIANNESS, MASKED_INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    xml, Parse,
};

impl Parse for MaskedIntRegNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), MASKED_INT_REG);
        let attr_base = node.parse(store);

        let register_base = node.parse(store);
        let bit_mask = node.parse(store);
        let sign = node.parse_if(SIGN, store).unwrap_or_default();
        let endianness = node.parse_if(ENDIANNESS, store).unwrap_or_default();
        let unit = node.parse_if(UNIT, store);
        let representation = node.parse_if(REPRESENTATION, store).unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED, store);

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
    use crate::{elem_type::BitMask, store::DefaultNodeStore};

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

        let mut store = DefaultNodeStore::new();
        let node: MaskedIntRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

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

        let mut store = DefaultNodeStore::new();
        let node: MaskedIntRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        debug_assert_eq!(node.bit_mask(), BitMask::Range { lsb: 3, msb: 7 });
    }
}
