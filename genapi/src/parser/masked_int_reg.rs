use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    MaskedIntRegNode,
};

use super::{
    elem_name::{ENDIANNESS, MASKED_INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    xml, Parse,
};

impl Parse for MaskedIntRegNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `MaskedIntRegNode`");
        debug_assert_eq!(node.tag_name(), MASKED_INT_REG);
        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let register_base = node.parse(node_builder, value_builder, cache_builder);

        let bit_mask = node.parse(node_builder, value_builder, cache_builder);
        let sign = node
            .parse_if(SIGN, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let endianness = node
            .parse_if(ENDIANNESS, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let unit = node.parse_if(UNIT, node_builder, value_builder, cache_builder);
        let representation = node
            .parse_if(REPRESENTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED, node_builder, value_builder, cache_builder);

        let node = Self {
            attr_base,
            register_base,
            bit_mask,
            sign,
            endianness,
            unit,
            representation,
            p_selected,
        };
        node.register_base
            .store_invalidators(node.attr_base.id, cache_builder);
        node
    }
}

#[cfg(test)]
mod tests {
    use crate::elem_type::BitMask;

    use super::{super::utils::tests::parse_default, *};

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

        let (node, ..): (MaskedIntRegNode, _, _, _) = parse_default(xml);

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

        let (node, ..): (MaskedIntRegNode, _, _, _) = parse_default(xml);
        debug_assert_eq!(node.bit_mask(), BitMask::Range { lsb: 3, msb: 7 });
    }
}
