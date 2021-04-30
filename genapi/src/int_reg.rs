use super::{
    elem_type::{Endianness, IntegerRepresentation, Sign},
    interface::{IInteger, IRegister, ISelector, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase},
    register_base::RegisterBase,
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct IntRegNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) register_base: RegisterBase,

    pub(crate) sign: Sign,
    pub(crate) endianness: Endianness,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
    pub(crate) p_selected: Vec<NodeId>,
}

impl IntRegNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    #[must_use]
    pub fn sign(&self) -> Sign {
        self.sign
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
    pub fn representation_elem(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

impl IInteger for IntRegNode {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let nid = self.node_base().id();
        let reg = self.register_base();

        let res = if let Some(cache) = cx.get_cached(nid) {
            let res = utils::int_from_slice(cache, self.endianness, self.sign)?;
            // Avoid a lifetime problem.
            reg.elem_base.verify_is_readable(device, store, cx)?;
            res
        } else {
            utils::int_from_slice(
                &reg.read_then_cache(nid, device, store, cx)?,
                self.endianness,
                self.sign,
            )?
        };
        Ok(res)
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let nid = self.node_base().id();
        cx.invalidate_cache_by(nid);

        let reg = self.register_base();
        let len = reg.length(device, store, cx)?;
        let mut buf = vec![0u8; len as usize];
        utils::bytes_from_int(value, &mut buf, self.endianness, self.sign)?;
        reg.write_then_cache(nid, &buf, device, store, cx)?;
        Ok(())
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        match self.sign {
            Sign::Signed => Ok(i64::MIN),
            Sign::Unsigned => Ok(0),
        }
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        Ok(i64::MAX)
    }

    fn inc_mode(&self, _: &impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        Ok(None)
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        Ok(None)
    }

    fn valid_value_set(&self, _: &impl NodeStore) -> &[i64] {
        &[]
    }

    fn representation(&self, _: &impl NodeStore) -> IntegerRepresentation {
        self.representation_elem()
    }

    fn unit(&self, _: &impl NodeStore) -> Option<&str> {
        self.unit_elem()
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        _: i64,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::AccessDenied(
            "can't set value to register's min elem".into(),
        ))
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        _: i64,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::AccessDenied(
            "can't set value to register's max elem".into(),
        ))
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base()
            .elem_base
            .is_readable(device, store, cx)
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base()
            .elem_base
            .is_writable(device, store, cx)
    }
}

impl IRegister for IntRegNode {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.register_base()
            .read_then_cache_with_buf(self.node_base().id(), buf, device, store, cx)
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.register_base()
            .write_then_cache(self.node_base().id(), buf, device, store, cx)
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

impl ISelector for IntRegNode {
    fn selecting_nodes(&self, _: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}
