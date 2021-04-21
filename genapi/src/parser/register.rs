use crate::{store::NodeStore, RegisterNode};

use super::{elem_name::REGISTER, xml, Parse};

impl Parse for RegisterNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert_eq!(node.tag_name(), REGISTER);

        let attr_base = node.parse(store);
        let register_base = node.parse(store);

        Self {
            attr_base,
            register_base,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        elem_type::{AccessMode, AddressKind, CachingMode, ImmOrPNode},
        store::DefaultNodeStore,
    };

    use super::*;

    #[test]
    fn test_register() {
        let xml = r#"
        <Register Name="TestNode">
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
          <PollingTime>300</PollingTime>
          <pInvalidator>ExposureTimeMode</pInvalidator>
          <Sign>Signed</Sign>
          <Endianess>BigEndian</Endianess>
          <Unit>Hz</Unit>
          <Representation>Logarithmic</Representation>
          <pSelected>SelectedNode</pSelected>
        </Register>
        "#;

        let mut store = DefaultNodeStore::new();
        let node: RegisterNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);
        let reg_base = node.register_base();

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
                assert_eq!(p_index.p_index(), store.id_by_name("IndexNode"));
            }
            _ => panic!(),
        }

        assert_eq!(reg_base.length(), &ImmOrPNode::Imm(4));
        assert_eq!(reg_base.access_mode(), AccessMode::RW);
        assert_eq!(reg_base.p_port(), "Device");
        assert_eq!(reg_base.cacheable(), CachingMode::WriteAround);
        assert_eq!(reg_base.polling_time().unwrap(), 300);
        assert_eq!(reg_base.p_invalidators().len(), 1);
    }
}
