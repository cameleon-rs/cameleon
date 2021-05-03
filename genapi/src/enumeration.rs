use super::{
    elem_type::ImmOrPNode,
    interface::{IEnumeration, ISelector},
    ivalue::IValue,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, IntegerId, NodeId, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
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
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<&EnumEntryNode> {
        self.elem_base.verify_is_readable(device, store, cx)?;

        let value = self.value.value(device, store, cx)?;
        self.entries(store)?
            .into_iter()
            .find(|ent| ent.value() == value)
            .ok_or_else(|| {
                GenApiError::invalid_node(
                    format!(
                        "no entry found corresponding to the current value of {}",
                        store.name_by_id(self.node_base().id()).unwrap()
                    )
                    .into(),
                )
            })
    }

    fn entries(&self, _: &impl NodeStore) -> GenApiResult<&[EnumEntryNode]> {
        Ok(&self.entries)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_entry_by_name<T: ValueStore, U: CacheStore>(
        &self,
        name: &str,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let ent_id = store.id_by_name(name).ok_or_else(|| {
            GenApiError::invalid_data(format! {"no `EnumEntryNode`: {} not found", name}.into())
        })?;

        let idx = self
            .entries(store)?
            .into_iter()
            .position(|ent| ent.node_base().id() == ent_id)
            .ok_or_else(|| {
                GenApiError::invalid_data(
                    format! {"no `EenumEntryNode`: {} not found in {}",
                    name,
                    store.name_by_id(self.node_base().id()).unwrap()}
                    .into(),
                )
            })?;

        self.set_entry_by_idx(idx, device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_entry_by_idx<T: ValueStore, U: CacheStore>(
        &self,
        idx: usize,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.elem_base.verify_is_writable(device, store, cx)?;
        let ent = self
            .entries(store)?
            .get(idx)
            .ok_or_else(|| GenApiError::invalid_data("entry index is out of range".into()))?;

        cx.invalidate_cache_by(self.node_base().id());
        let value = ent.value();
        self.value.set_value(value, device, store, cx)
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
    fn selecting_nodes(&self, _: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}
