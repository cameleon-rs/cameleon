use crate::{
    store::{NodeStore, ValueStore},
    IntRegNode,
};

use super::{
    elem_name::{ENDIANNESS, INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    xml, Parse,
};

impl Parse for IntRegNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), INT_REG);

        let attr_base = node.parse(node_store, value_store);
        let register_base = node.parse(node_store, value_store);

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
        elem_type::{Endianness, IntegerRepresentation, Sign},
        store::{DefaultNodeStore, DefaultValueStore},
    };

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

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: IntRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

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
