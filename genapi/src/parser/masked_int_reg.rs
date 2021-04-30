use crate::{
    store::{ValueStore, WritableNodeStore},
    MaskedIntRegNode,
};

use super::{
    elem_name::{ENDIANNESS, MASKED_INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    xml, Parse,
};

impl Parse for MaskedIntRegNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), MASKED_INT_REG);
        let attr_base = node.parse(node_store, value_store);

        let register_base = node.parse(node_store, value_store);
        let bit_mask = node.parse(node_store, value_store);
        let sign = node
            .parse_if(SIGN, node_store, value_store)
            .unwrap_or_default();
        let endianness = node
            .parse_if(ENDIANNESS, node_store, value_store)
            .unwrap_or_default();
        let unit = node.parse_if(UNIT, node_store, value_store);
        let representation = node
            .parse_if(REPRESENTATION, node_store, value_store)
            .unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED, node_store, value_store);

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
    use crate::{
        elem_type::BitMask,
        store::{DefaultNodeStore, DefaultValueStore},
    };

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

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: MaskedIntRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

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

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: MaskedIntRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        debug_assert_eq!(node.bit_mask(), BitMask::Range { lsb: 3, msb: 7 });
    }
}
