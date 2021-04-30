use super::{
    elem_type::{DisplayNotation, FloatRepresentation, ImmOrPNode, ValueKind},
    interface::{IFloat, IncrementMode},
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
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

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

impl IFloat for FloatNode {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        todo!()
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo! {}
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        todo!()
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        todo!()
    }

    fn inc_mode(&self, store: &impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        todo!()
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        todo!()
    }

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: &impl NodeStore) -> &[f64] {
        todo!()
    }

    fn representation(&self, store: &impl NodeStore) -> FloatRepresentation {
        todo!()
    }

    fn unit(&self, store: &impl NodeStore) -> Option<&str> {
        todo!()
    }

    fn display_notation(&self, store: &impl NodeStore) -> DisplayNotation {
        todo!()
    }

    fn display_precision(&self, store: &impl NodeStore) -> i64 {
        todo! {}
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        todo!()
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        todo!()
    }
}
