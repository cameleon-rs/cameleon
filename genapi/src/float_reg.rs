use super::{
    elem_type::{DisplayNotation, Endianness, FloatRepresentation},
    interface::{IFloat, INode, IRegister, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase},
    store::{CacheStore, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, RegisterBase, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct FloatRegNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) register_base: RegisterBase,

    pub(crate) endianness: Endianness,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
}

impl FloatRegNode {
    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    #[must_use]
    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    #[must_use]
    pub fn unit_elem(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation_elem(&self) -> FloatRepresentation {
        self.representation
    }

    #[must_use]
    pub fn display_notation_elem(&self) -> DisplayNotation {
        self.display_notation
    }

    #[must_use]
    pub fn display_precision_elem(&self) -> i64 {
        self.display_precision
    }
}

impl INode for FloatRegNode {
    fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    fn streamable(&self) -> bool {
        self.register_base().streamable()
    }
}

impl IFloat for FloatRegNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        let nid = self.node_base().id();
        let reg = self.register_base();

        reg.with_cache_or_read(nid, device, store, cx, |data| {
            utils::float_from_slice(data, self.endianness)
        })
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let nid = self.node_base().id();
        cx.invalidate_cache_by(nid);

        let reg = self.register_base();
        let len = reg.length(device, store, cx)?;
        let mut buf = vec![0; len as usize];
        utils::bytes_from_float(value, &mut buf, self.endianness)?;
        reg.write_and_cache(nid, &buf, device, store, cx)?;
        Ok(())
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        Ok(f64::MIN)
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        Ok(f64::MAX)
    }

    fn inc_mode(&self, _: &impl NodeStore) -> Option<IncrementMode> {
        None
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        Ok(None)
    }

    fn representation(&self, _: &impl NodeStore) -> FloatRepresentation {
        self.representation
    }

    fn unit(&self, _: &impl NodeStore) -> Option<&str> {
        self.unit_elem()
    }

    fn display_notation(&self, _: &impl NodeStore) -> DisplayNotation {
        self.display_notation
    }

    fn display_precision(&self, _: &impl NodeStore) -> i64 {
        self.display_precision
    }

    #[tracing::instrument(skip(self, store),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        _: f64,
        _: &mut impl Device,
        store: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::not_writable())
    }

    #[tracing::instrument(skip(self, store),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        _: f64,
        _: &mut impl Device,
        store: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::not_writable())
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base().is_readable(device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base().is_writable(device, store, cx)
    }
}

impl IRegister for FloatRegNode {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let address = self.address(device, store, cx)?;
        let length = self.length(device, store, cx)?;
        self.register_base().read_and_cache(
            self.node_base().id(),
            address,
            length,
            buf,
            device,
            store,
            cx,
        )
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.register_base()
            .write_and_cache(self.node_base().id(), buf, device, store, cx)
    }

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.register_base().address(device, store, cx)
    }

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.register_base().length(device, store, cx)
    }
}
