use super::{elem_type::*, node_base::*, register_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct IntRegNode {
    register_base: RegisterBase,

    sign: register_node_elem::Sign,

    endianness: register_node_elem::Endianness,

    unit: Option<String>,

    representation: IntegerRepresentation,

    p_selected: Vec<String>,
}

impl IntRegNode {
    pub fn node_base(&self) -> NodeBase {
        self.register_base.node_base()
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    pub fn sign(&self) -> register_node_elem::Sign {
        self.sign
    }

    pub fn endianness(&self) -> register_node_elem::Endianness {
        self.endianness
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }
}

impl Parse for IntRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "IntReg");

        let register_base = node.parse();

        let sign = node.parse_if("Sign").unwrap_or_default();

        let endianness = node.parse_if("Endianess").unwrap_or_default();

        let unit = node.parse_if("Unit");

        let representation = node.parse_if("Representation").unwrap_or_default();

        let mut p_selected = vec![];
        while let Some(selected) = node.parse_if("pSelected") {
            p_selected.push(selected);
        }

        Self {
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
    use super::*;
    use crate::register_node_elem::*;

    #[test]
    fn test_int_reg() {
        let xml = r#"
        <IntReg Name="TestNode">
          <Streamable>No</Streamable>
          <Address>0x10000</Address>
          <IntSwissKnife Name="Testnode">
              <pVariable Name="Var1">pValue1</pVariable>
              <pVariable Name="Var2">pValue2</pVariable>
              <Constant Name="Const">10</Constant>
              <Expression Name="ConstBy2">2.0*Const</Expression>
              <Formula>Var1+Var2+ConstBy2</Formula>
          </IntSwissKnife>
          <pAddress>pAddress</pAddress>
          <pIndex Offset="10">IndexNode</pIndex>
          <Length>4</Length>
          <AccessMode>RW</AccessMode>
          <pPort>Device</pPort>
          <Cachable>WriteAround</Cachable>
          <pInvalidator>ExposureTimeMode</pInvalidator>
          <Sign>Signed</Sign>
          <Endianess>BigEndian</Endianess>
          <Unit>Hz</Unit>
          <Representation>Logarithmic</Representation>
          <pSelected>SelectedNode</pSelected>
        </IntReg>
        "#;

        let node: IntRegNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        let reg_base = node.register_base();
        assert!(!reg_base.streamable());

        let address_kinds = reg_base.address_kinds();
        assert_eq!(address_kinds.len(), 4);
        assert!(matches!(
            address_kinds[0],
            AddressKind::Address(ImmOrPNode::Imm(0x10000))
        ));
        assert!(matches!(address_kinds[1], AddressKind::IntSwissKnife(_)));
        assert!(matches!(
            address_kinds[2],
            AddressKind::Address(ImmOrPNode::PNode(_))
        ));
        match &address_kinds[3] {
            AddressKind::PIndex(p_index) => {
                assert!(matches!(p_index.offset().unwrap(), ImmOrPNode::Imm(10)));
                assert_eq!(p_index.p_index(), "IndexNode");
            }
            _ => panic!(),
        }
        assert_eq!(reg_base.p_port(), "Device");
        assert_eq!(reg_base.cacheable(), CachingMode::WriteAround);

        assert_eq!(node.sign(), Sign::Signed);
        assert_eq!(node.endianness(), Endianness::BE);
        assert_eq!(node.p_selected().len(), 1);
    }
}
