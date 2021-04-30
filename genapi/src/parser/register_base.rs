use crate::{
    elem_type::AccessMode,
    store::{WritableNodeStore, ValueStore},
    RegisterBase,
};

use super::{
    elem_name::{
        ACCESS_MODE, ADDRESS, CACHEABLE, INT_SWISS_KNIFE, POLLING_TIME, P_ADDRESS, P_INDEX,
        P_INVALIDATOR, STREAMABLE,
    },
    xml, Parse,
};

impl Parse for RegisterBase {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let elem_base = node.parse(node_store, value_store);

        let streamable = node
            .parse_if(STREAMABLE, node_store, value_store)
            .unwrap_or_default();
        let mut address_kinds = vec![];
        while let Some(addr_kind) = node
            .parse_if(ADDRESS, node_store, value_store)
            .or_else(|| node.parse_if(INT_SWISS_KNIFE, node_store, value_store))
            .or_else(|| node.parse_if(P_ADDRESS, node_store, value_store))
            .or_else(|| node.parse_if(P_INDEX, node_store, value_store))
        {
            address_kinds.push(addr_kind);
        }
        let length = node.parse(node_store, value_store);
        let access_mode = node
            .parse_if(ACCESS_MODE, node_store, value_store)
            .unwrap_or(AccessMode::RO);
        let p_port = node.parse(node_store, value_store);
        let cacheable = node
            .parse_if(CACHEABLE, node_store, value_store)
            .unwrap_or_default();
        let polling_time = node.parse_if(POLLING_TIME, node_store, value_store);
        let p_invalidators = node.parse_while(P_INVALIDATOR, node_store, value_store);

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
