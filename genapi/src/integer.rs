use super::{
    elem_type::{ImmOrPNode, IntegerRepresentation, ValueKind},
    interface::IncrementMode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, IntegerId, NodeId, NodeStore, ValueStore},
    Device, GenApiResult, IInteger, ValueCtxt,
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
    pub fn min(&self) -> ImmOrPNode<IntegerId> {
        self.min
    }

    #[must_use]
    pub fn max(&self) -> ImmOrPNode<IntegerId> {
        self.max
    }

    #[must_use]
    pub fn inc(&self) -> ImmOrPNode<i64> {
        self.inc
    }

    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

impl IInteger for IntegerNode {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        todo!()
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        todo!()
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        todo! {}
    }

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        todo!()
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        todo!()
    }

    fn valid_value_set(&self, store: impl NodeStore) -> &[i64] {
        todo!()
    }

    fn representation(&self, store: impl NodeStore) -> IntegerRepresentation {
        todo!()
    }

    fn unit(&self, store: impl NodeStore) -> Option<&str> {
        todo!()
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }
}
