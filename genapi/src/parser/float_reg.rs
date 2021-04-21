use crate::{store::NodeStore, FloatRegNode};

use super::{
    elem_name::{DISPLAY_NOTATION, DISPLAY_PRECISION, ENDIANNESS, FLOAT_REG, REPRESENTATION, UNIT},
    xml, Parse,
};

impl Parse for FloatRegNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), FLOAT_REG);

        let attr_base = node.parse(store);
        let register_base = node.parse(store);

        let endianness = node.parse_if(ENDIANNESS, store).unwrap_or_default();
        let unit = node.parse_if(UNIT, store);
        let representation = node.parse_if(REPRESENTATION, store).unwrap_or_default();
        let display_notation = node.parse_if(DISPLAY_NOTATION, store).unwrap_or_default();
        let display_precision = node.parse_if(DISPLAY_PRECISION, store).unwrap_or(6);

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

    use crate::{
        elem_type::{DisplayNotation, Endianness, FloatRepresentation},
        store::DefaultNodeStore,
    };

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

        let mut store = DefaultNodeStore::new();
        let node: FloatRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.unit().unwrap(), "Hz");
        assert_eq!(node.representation(), FloatRepresentation::Linear);
        assert_eq!(node.display_notation(), DisplayNotation::Fixed);
        assert_eq!(node.display_precision(), 10);
    }
}
