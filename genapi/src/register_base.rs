use super::{
    elem_type::{AccessMode, AddressKind, CachingMode, ImmOrPNode},
    interface::IRegister,
    node_base::NodeElementBase,
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct RegisterBase {
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) address_kinds: Vec<AddressKind>,
    pub(crate) length: ImmOrPNode<i64>,
    pub(crate) access_mode: AccessMode,
    pub(crate) p_port: NodeId,
    pub(crate) cacheable: CachingMode,
    pub(crate) polling_time: Option<u64>,
    pub(crate) p_invalidators: Vec<NodeId>,
}

impl RegisterBase {
    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn address_kinds(&self) -> &[AddressKind] {
        &self.address_kinds
    }

    #[must_use]
    pub fn length_elem(&self) -> &ImmOrPNode<i64> {
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

impl IRegister for RegisterBase {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        todo!()
    }

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        todo!()
    }
}
