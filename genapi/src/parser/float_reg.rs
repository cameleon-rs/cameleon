use crate::{
    store::{WritableNodeStore, ValueStore},
    FloatRegNode,
};

use super::{
    elem_name::{DISPLAY_NOTATION, DISPLAY_PRECISION, ENDIANNESS, FLOAT_REG, REPRESENTATION, UNIT},
    xml, Parse,
};

impl Parse for FloatRegNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), FLOAT_REG);

        let attr_base = node.parse(node_store, value_store);
        let register_base = node.parse(node_store, value_store);

        let endianness = node
            .parse_if(ENDIANNESS, node_store, value_store)
            .unwrap_or_default();
        let unit = node.parse_if(UNIT, node_store, value_store);
        let representation = node
            .parse_if(REPRESENTATION, node_store, value_store)
            .unwrap_or_default();
        let display_notation = node
            .parse_if(DISPLAY_NOTATION, node_store, value_store)
            .unwrap_or_default();
        let display_precision = node
            .parse_if(DISPLAY_PRECISION, node_store, value_store)
            .unwrap_or(6);

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
        store::{DefaultNodeStore, DefaultValueStore},
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

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: FloatRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.unit_elem().unwrap(), "Hz");
        assert_eq!(node.representation_elem(), FloatRepresentation::Linear);
        assert_eq!(node.display_notation_elem(), DisplayNotation::Fixed);
        assert_eq!(node.display_precision_elem(), 10);
    }
}
