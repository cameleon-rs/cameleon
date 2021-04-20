use crate::{store::NodeStore, IntRegNode};

use super::{
    elem_name::{ENDIANNESS, INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    xml, Parse,
};

impl Parse for IntRegNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), INT_REG);

        let attr_base = node.parse(store);
        let register_base = node.parse(store);

        let sign = node.parse_if(SIGN, store).unwrap_or_default();
        let endianness = node.parse_if(ENDIANNESS, store).unwrap_or_default();
        let unit = node.parse_if(UNIT, store);
        let representation = node.parse_if(REPRESENTATION, store).unwrap_or_default();
        let p_selected = node.parse_while(P_SELECTED, store);

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
    use crate::elem_type::{Endianness, IntegerRepresentation, Sign};

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

        let mut store = NodeStore::new();
        let node: IntRegNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.sign(), Sign::Signed);
        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.unit().unwrap(), "Hz");
        assert_eq!(node.representation(), IntegerRepresentation::Logarithmic);
        assert_eq!(node.p_selected().len(), 1);
    }
}
