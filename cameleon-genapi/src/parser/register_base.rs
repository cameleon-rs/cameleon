use super::{
    elem_name::{
        ACCESS_MODE, ADDRESS, CACHEABLE, INT_SWISS_KNIFE, POLLING_TIME, P_ADDRESS, P_INDEX,
        P_INVALIDATOR, STREAMABLE,
    },
    elem_type::{register_node_elem, AccessMode, CachingMode, ImmOrPNode},
    node_base::NodeElementBase,
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct RegisterBase {
    pub(super) elem_base: NodeElementBase,

    pub(super) streamable: bool,
    pub(super) address_kinds: Vec<register_node_elem::AddressKind>,
    pub(super) length: ImmOrPNode<i64>,
    pub(super) access_mode: AccessMode,
    pub(super) p_port: NodeId,
    pub(super) cacheable: CachingMode,
    pub(super) polling_time: Option<u64>,
    pub(super) p_invalidators: Vec<NodeId>,
}

impl RegisterBase {
    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn address_kinds(&self) -> &[register_node_elem::AddressKind] {
        &self.address_kinds
    }

    #[must_use]
    pub fn length(&self) -> &ImmOrPNode<i64> {
        &self.length
    }

    #[must_use]
    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    #[must_use]
    pub fn p_port(&self) -> NodeId {
        self.p_port
    }

    #[must_use]
    pub fn cacheable(&self) -> CachingMode {
        self.cacheable
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }
}

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
