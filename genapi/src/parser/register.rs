use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    RegisterNode,
};

use super::{elem_name::REGISTER, xml, Parse};

impl Parse for RegisterNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `RegisterNode`");
        debug_assert_eq!(node.tag_name(), REGISTER);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let register_base = node.parse(node_builder, value_builder, cache_builder);

        let node = Self {
            attr_base,
            register_base,
        };
        node.register_base
            .store_invalidators(node.attr_base.id, cache_builder);
        node
    }
}

#[cfg(test)]
mod tests {
    use crate::elem_type::{AccessMode, AddressKind, CachingMode, ImmOrPNode};

    use super::{super::utils::tests::parse_default, *};

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

        let (node, mut node_builder, ..): (RegisterNode, _, _, _) = parse_default(xml);
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
                assert_eq!(p_index.p_index(), node_builder.get_or_intern("IndexNode"));
            }
            _ => panic!(),
        }

        assert_eq!(reg_base.length_elem(), &ImmOrPNode::Imm(4));
        assert_eq!(reg_base.access_mode(), AccessMode::RW);
        assert_eq!(reg_base.cacheable(), CachingMode::WriteAround);
        assert_eq!(reg_base.polling_time().unwrap(), 300);
        assert_eq!(reg_base.p_invalidators().len(), 1);
    }
}
