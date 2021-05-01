use super::{
    elem_type::{ImmOrPNode, IntegerRepresentation, ValueKind},
    interface::{IInteger, ISelector, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, IntegerId, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct IntegerNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) value_kind: ValueKind<IntegerId>,
    pub(crate) min: ImmOrPNode<IntegerId>,
    pub(crate) max: ImmOrPNode<IntegerId>,
    pub(crate) inc: ImmOrPNode<i64>,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
    pub(crate) p_selected: Vec<NodeId>,
}

impl IntegerNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn value_kind(&self) -> &ValueKind<IntegerId> {
        &self.value_kind
    }

    #[must_use]
    pub fn min_elem(&self) -> ImmOrPNode<IntegerId> {
        self.min
    }

    #[must_use]
    pub fn max_elem(&self) -> ImmOrPNode<IntegerId> {
        self.max
    }

    #[must_use]
    pub fn inc_elem(&self) -> ImmOrPNode<i64> {
        self.inc
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

impl IInteger for IntegerNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.elem_base.verify_is_readable(device, store, cx)?;
        self.value_kind().value(device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.elem_base.verify_is_writable(device, store, cx)?;
        cx.invalidate_cache_by(self.node_base().id());

        let min = self.min(device, store, cx)?;
        let max = self.max(device, store, cx)?;
        utils::verify_value_in_range(value, min, max)?;

        self.value_kind().set_value(value, device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.min.value(device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.max.value(device, store, cx)
    }

    fn inc_mode(&self, _: &impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        Ok(Some(IncrementMode::FixedIncrement))
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        Some(self.inc.value(device, store, cx)).transpose()
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

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.min.set_value(value, device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.max.set_value(value, device, store, cx)
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.elem_base.is_readable(device, store, cx)
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.elem_base.is_writable(device, store, cx)
    }
}

impl ISelector for IntegerNode {
    fn selecting_nodes(&self, _: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}
