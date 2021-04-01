use super::{
    elem_name::{ENDIANNESS, INT_REG, P_SELECTED, REPRESENTATION, SIGN, UNIT},
    elem_type::{register_node_elem, IntegerRepresentation},
    node_base::{NodeAttributeBase, NodeBase},
    node_store::{NodeId, NodeStore},
    register_base::RegisterBase,
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct IntRegNode {
    attr_base: NodeAttributeBase,
    register_base: RegisterBase,

    sign: register_node_elem::Sign,
    endianness: register_node_elem::Endianness,
    unit: Option<String>,
    representation: IntegerRepresentation,
    p_selected: Vec<NodeId>,
}

impl IntRegNode {
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
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

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
    use super::super::register_node_elem::{Endianness, Sign};

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
