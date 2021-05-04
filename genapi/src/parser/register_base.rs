use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    elem_type::AccessMode,
    store::NodeId,
    RegisterBase,
};

use super::{
    elem_name::{
        ACCESS_MODE, ADDRESS, CACHEABLE, INT_SWISS_KNIFE, POLLING_TIME, P_ADDRESS, P_INDEX,
        P_INVALIDATOR, STREAMABLE,
    },
    xml, Parse,
};

impl RegisterBase {
    pub(super) fn store_invalidators(
        &self,
        target: NodeId,
        cache_builder: &mut impl CacheStoreBuilder,
    ) {
        for invalidator in &self.p_invalidators {
            cache_builder.store_invalidator(*invalidator, target);
        }
    }
}

impl Parse for RegisterBase {
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let mut address_kinds = vec![];
        while let Some(addr_kind) = node
            .parse_if(ADDRESS, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(INT_SWISS_KNIFE, node_builder, value_builder, cache_builder))
            .or_else(|| node.parse_if(P_ADDRESS, node_builder, value_builder, cache_builder))
            .or_else(|| node.parse_if(P_INDEX, node_builder, value_builder, cache_builder))
        {
            address_kinds.push(addr_kind);
        }
        let length = node.parse(node_builder, value_builder, cache_builder);
        let access_mode = node
            .parse_if(ACCESS_MODE, node_builder, value_builder, cache_builder)
            .unwrap_or(AccessMode::RO);
        let p_port = node.parse(node_builder, value_builder, cache_builder);
        let cacheable = node
            .parse_if(CACHEABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let polling_time = node.parse_if(POLLING_TIME, node_builder, value_builder, cache_builder);
        let p_invalidators =
            node.parse_while(P_INVALIDATOR, node_builder, value_builder, cache_builder);

        Self {
            elem_base,
            streamable,
            address_kinds,
            length,
            access_mode,
            p_port,
            cacheable,
            polling_time,
            p_invalidators,
        }
    }
}
