use super::{
    elem_type::ImmOrPNode,
    interface::{IEnumeration, INode, ISelector},
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

impl INode for EnumerationNode {
    fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        self.streamable
    }
}

impl IEnumeration for EnumerationNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn current_value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.value.value(device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<&EnumEntryNode> {
        let value = self.value.value(device, store, cx)?;
        self.entries(store)
            .iter()
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

    fn entries(&self, _: &impl NodeStore) -> &[EnumEntryNode] {
        &self.entries
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
        let value = self
            .entries(store)
            .iter()
            .find(|ent| ent.name() == name)
            .ok_or_else(|| {
                GenApiError::invalid_data(
                    format! {"no `EenumEntryNode`: `{}` not found in `{}`",
                    name,
                    store.name_by_id(self.node_base().id()).unwrap()}
                    .into(),
                )
            })?
            .value();

        self.set_entry_by_value(value, device, store, cx)
    }

    fn set_entry_by_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        if !self.entries(store).iter().any(|ent| ent.value() == value) {
            return Err(GenApiError::invalid_data(
                format!("not found entry with the value `{}`", value).into(),
            ));
        };
        cx.invalidate_cache_by(self.node_base().id());
        self.value.set_value(value, device, store, cx)
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
            && self.value.is_readable(device, store, cx)?)
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
            && self.value.is_writable(device, store, cx)?)
    }
}

impl ISelector for EnumerationNode {
    fn selecting_nodes(&self, _: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}

#[derive(Debug, Clone)]
pub struct EnumEntryNode {
    pub(crate) name: String,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) value: i64,
    pub(crate) numeric_value: Option<f64>,
    pub(crate) symbolic: Option<String>,
    pub(crate) is_self_clearing: bool,
}

impl EnumEntryNode {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn numeric_value(&self) -> f64 {
        self.numeric_value.unwrap_or_else(|| self.value as f64)
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
