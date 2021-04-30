use super::{
    elem_type::ImmOrPNode,
    interface::{IEnumeration, ISelector},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, IntegerId, NodeId, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct EnumerationNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) entries: Vec<EnumEntryNode>,
    pub(crate) value: ImmOrPNode<IntegerId>,
    pub(crate) p_selected: Vec<NodeId>,
    pub(crate) polling_time: Option<u64>,
}

impl EnumerationNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn entries_elem(&self) -> &[EnumEntryNode] {
        &self.entries
    }

    #[must_use]
    pub fn value_elem(&self) -> ImmOrPNode<IntegerId> {
        self.value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

impl IEnumeration for EnumerationNode {
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<&EnumEntryNode> {
        todo!()
    }

    fn entries(&self, store: &impl NodeStore) -> GenApiResult<&[EnumEntryNode]> {
        todo!()
    }

    fn set_entry_by_name<T: ValueStore, U: CacheStore>(
        &self,
        name: &str,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn set_entry_by_idx<T: ValueStore, U: CacheStore>(
        &self,
        idx: usize,
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

#[derive(Debug, Clone)]
pub struct EnumEntryNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) value: i64,
    pub(crate) numeric_values: Vec<f64>,
    pub(crate) symbolic: Option<String>,
    pub(crate) is_self_clearing: bool,
}

impl EnumEntryNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }

    #[must_use]
    pub fn numeric_values(&self) -> &[f64] {
        &self.numeric_values
    }

    #[must_use]
    pub fn symbolic(&self) -> Option<&str> {
        self.symbolic.as_deref()
    }

    pub fn set_symbolic(&mut self, s: String) {
        self.symbolic = Some(s)
    }

    #[must_use]
    pub fn is_self_clearing(&self) -> bool {
        self.is_self_clearing
    }
}

impl ISelector for EnumerationNode {
    fn selecting_nodes(&self, store: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}
