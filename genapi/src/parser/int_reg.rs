/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    IntRegNode,
};

use super::{
    elem_name::{ENDIANNESS, INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    xml, Parse,
};

impl Parse for IntRegNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `IntRegNode`");
        debug_assert_eq!(node.tag_name(), INT_REG);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let register_base = node.parse(node_builder, value_builder, cache_builder);

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
    use crate::elem_type::{Endianness, IntegerRepresentation, Sign};

    use super::{super::utils::tests::parse_default, *};

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

        let (node, ..): (IntRegNode, _, _, _) = parse_default(xml);

        assert_eq!(node.sign(), Sign::Signed);
        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.unit_elem().unwrap(), "Hz");
        assert_eq!(
            node.representation_elem(),
            IntegerRepresentation::Logarithmic
        );
        assert_eq!(node.p_selected().len(), 1);
    }
}
