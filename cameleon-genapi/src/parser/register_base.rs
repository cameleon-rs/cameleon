use crate::{elem_type::AccessMode, node_store::NodeStore, RegisterBase};

use super::{
    elem_name::{
        ACCESS_MODE, ADDRESS, CACHEABLE, INT_SWISS_KNIFE, POLLING_TIME, P_ADDRESS, P_INDEX,
        P_INVALIDATOR, STREAMABLE,
    },
    xml, Parse,
};

impl Parse for RegisterBase {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        let elem_base = node.parse(store);

        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let mut address_kinds = vec![];
        while let Some(addr_kind) = node
            .parse_if(ADDRESS, store)
            .or_else(|| node.parse_if(INT_SWISS_KNIFE, store))
            .or_else(|| node.parse_if(P_ADDRESS, store))
            .or_else(|| node.parse_if(P_INDEX, store))
        {
            address_kinds.push(addr_kind);
        }
        let length = node.parse(store);
        let access_mode = node.parse_if(ACCESS_MODE, store).unwrap_or(AccessMode::RO);
        let p_port = node.parse(store);
        let cacheable = node.parse_if(CACHEABLE, store).unwrap_or_default();
        let polling_time = node.parse_if(POLLING_TIME, store);
        let p_invalidators = node.parse_while(P_INVALIDATOR, store);

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
