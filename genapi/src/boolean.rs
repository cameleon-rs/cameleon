use super::{
    elem_type::ImmOrPNode,
    interface::{IBoolean, ISelector},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{BooleanId, CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct BooleanNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) value: ImmOrPNode<BooleanId>,
    pub(crate) on_value: Option<i64>,
    pub(crate) off_value: Option<i64>,
    pub(crate) p_selected: Vec<NodeId>,
}

impl BooleanNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn value_elem(&self) -> ImmOrPNode<BooleanId> {
        self.value
    }

    #[must_use]
    pub fn on_value(&self) -> Option<i64> {
        self.on_value
    }

    #[must_use]
    pub fn off_value(&self) -> Option<i64> {
        self.off_value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

impl IBoolean for BooleanNode {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        todo!()
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: bool,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }
}

impl ISelector for BooleanNode {
    fn selecting_nodes(&self, store: impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}
