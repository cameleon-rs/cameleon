use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    FloatRegNode,
};

use super::{
    elem_name::{DISPLAY_NOTATION, DISPLAY_PRECISION, ENDIANNESS, FLOAT_REG, REPRESENTATION, UNIT},
    xml, Parse,
};

impl Parse for FloatRegNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `FloatRegNode`");
        debug_assert_eq!(node.tag_name(), FLOAT_REG);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let register_base = node.parse(node_builder, value_builder, cache_builder);

        let endianness = node
            .parse_if(ENDIANNESS, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let unit = node.parse_if(UNIT, node_builder, value_builder, cache_builder);
        let representation = node
            .parse_if(REPRESENTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let display_notation = node
            .parse_if(DISPLAY_NOTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let display_precision = node
            .parse_if(
                DISPLAY_PRECISION,
                node_builder,
                value_builder,
                cache_builder,
            )
            .unwrap_or(6);

        let node = Self {
            attr_base,
            register_base,
            endianness,
            unit,
            representation,
            display_notation,
            display_precision,
        };
        node.register_base
            .store_invalidators(node.attr_base.id, cache_builder);
        node
    }
}

#[cfg(test)]
mod tests {
    use super::{super::utils::tests::parse_default, *};

    use crate::elem_type::{DisplayNotation, Endianness, FloatRepresentation};

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

        let (node, ..): (FloatRegNode, _, _, _) = parse_default(xml);

        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.unit_elem().unwrap(), "Hz");
        assert_eq!(node.representation_elem(), FloatRepresentation::Linear);
        assert_eq!(node.display_notation_elem(), DisplayNotation::Fixed);
        assert_eq!(node.display_precision_elem(), 10);
    }
}
