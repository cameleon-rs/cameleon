use super::{
    elem_type::{AccessMode, AddressKind, CachingMode, Endianness, ImmOrPNode, Sign},
    interface::{IPort, IRegister},
    node_base::NodeElementBase,
    store::{CacheStore, NodeId, NodeStore, ValueData, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
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

    pub(super) fn cache<T: ValueStore, U: CacheStore>(
        &self,
        nid: NodeId,
        data: impl Into<ValueData>,
        cx: &mut ValueCtxt<T, U>,
        on_read: bool,
    ) {
        match (self.cacheable, on_read) {
            (CachingMode::WriteThrough, _) => cx.cache_data(nid, data),
            (CachingMode::WriteAround, true) => cx.cache_data(nid, data),
            _ => {}
        }
    }

    pub(super) fn alloc_read<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Vec<u8>> {
        let len = self.length(device, store, cx)? as usize;
        let mut buf = vec![0; len];
        self.read(&mut buf, device, store, cx)?;
        Ok(buf)
    }
}

impl IRegister for RegisterBase {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.elem_base.verify_is_readable(device, store, cx)?;
        if buf.len() != self.length(device, store, cx)? as usize {
            return Err(GenApiError::InvalidBuffer(
                "given buffer length doesn't same as the register length".into(),
            ));
        }

        let addr = self.address(device, store, cx)?;
        self.p_port
            .expect_iport_kind(store)?
            .read(addr, buf, device, store, cx)
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.elem_base.verify_is_writable(device, store, cx)?;
        if buf.len() != self.length(device, store, cx)? as usize {
            return Err(GenApiError::InvalidBuffer(
                "given buffer length doesn't same as the register length".into(),
            ));
        }

        let addr = self.address(device, store, cx)?;
        self.p_port
            .expect_iport_kind(store)?
            .write(addr, buf, device, store, cx)
    }

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let mut address = 0;
        for addr_kind in self.address_kinds() {
            address += addr_kind.value(device, store, cx)?;
        }
        Ok(address)
    }

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.length_elem().value(device, store, cx)
    }
}
