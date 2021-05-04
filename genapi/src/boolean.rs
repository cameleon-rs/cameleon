use super::{
    elem_type::ImmOrPNode,
    interface::{IBoolean, IInteger, ISelector},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{BooleanId, CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct BooleanNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) value: ImmOrPNode<BooleanId>,
    pub(crate) on_value: i64,
    pub(crate) off_value: i64,
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
    pub fn on_value(&self) -> i64 {
        self.on_value
    }

    #[must_use]
    pub fn off_value(&self) -> i64 {
        self.off_value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

impl IBoolean for BooleanNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.elem_base.verify_is_readable(device, store, cx)?;
        match self.value {
            ImmOrPNode::Imm(vid) => Ok(cx.value_store().boolean_value(vid).unwrap()),
            ImmOrPNode::PNode(nid) => {
                let value = nid.expect_iinteger_kind(store)?.value(device, store, cx)?;
                if value == self.on_value {
                    Ok(true)
                } else if value == self.off_value {
                    Ok(false)
                } else {
                    Err(GenApiError::invalid_node(
                        "the internal integer value cannot be interpreted as boolean".into(),
                    ))
                }
            }
        }
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: bool,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.elem_base.verify_is_writable(device, store, cx)?;
        cx.invalidate_cache_by(self.node_base().id());
        match self.value {
            ImmOrPNode::Imm(vid) => {
                cx.value_store_mut().update(vid, value);
                Ok(())
            }
            ImmOrPNode::PNode(nid) => {
                let value = if value { self.on_value } else { self.off_value };
                nid.expect_iinteger_kind(store)?
                    .set_value(value, device, store, cx)
            }
        }
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

impl ISelector for BooleanNode {
    fn selecting_nodes(&self, _store: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}
