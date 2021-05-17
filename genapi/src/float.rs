use super::{
    elem_type::{DisplayNotation, FloatRepresentation, ImmOrPNode, ValueKind},
    interface::{IFloat, INode, IncrementMode},
    ivalue::IValue,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, FloatId, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct FloatNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) value_kind: ValueKind<FloatId>,
    pub(crate) min: ImmOrPNode<FloatId>,
    pub(crate) max: ImmOrPNode<FloatId>,
    pub(crate) inc: Option<ImmOrPNode<f64>>,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
}

impl FloatNode {
    #[must_use]
    pub fn value_kind(&self) -> &ValueKind<FloatId> {
        &self.value_kind
    }

    #[must_use]
    pub fn min_elem(&self) -> ImmOrPNode<FloatId> {
        self.min
    }

    #[must_use]
    pub fn max_elem(&self) -> ImmOrPNode<FloatId> {
        self.max
    }

    #[must_use]
    pub fn inc_elem(&self) -> Option<&ImmOrPNode<f64>> {
        self.inc.as_ref()
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

impl INode for FloatNode {
    fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        self.streamable
    }
}

impl IFloat for FloatNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        self.value_kind.value(device, store, cx)
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
        cx.invalidate_cache_by(self.node_base().id());
        self.value_kind.set_value(value, device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
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
    ) -> GenApiResult<f64> {
        self.max.value(device, store, cx)
    }

    fn inc_mode(&self, _store: &impl NodeStore) -> Option<IncrementMode> {
        Some(IncrementMode::FixedIncrement)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        self.inc.map(|n| n.value(device, store, cx)).transpose()
    }

    fn representation(&self, _store: &impl NodeStore) -> FloatRepresentation {
        self.representation_elem()
    }

    fn unit(&self, _store: &impl NodeStore) -> Option<&str> {
        self.unit_elem()
    }

    fn display_notation(&self, _store: &impl NodeStore) -> DisplayNotation {
        self.display_notation_elem()
    }

    fn display_precision(&self, _store: &impl NodeStore) -> i64 {
        self.display_precision_elem()
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
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
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.max.set_value(value, device, store, cx)
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
        Ok(self.elem_base.is_readable(device, store, cx)?
            && self.value_kind.is_readable(device, store, cx)?)
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
        Ok(self.elem_base.is_writable(device, store, cx)?
            && self.value_kind.is_writable(device, store, cx)?)
    }
}
