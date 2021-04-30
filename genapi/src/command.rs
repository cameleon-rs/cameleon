use super::{
    elem_type::ImmOrPNode,
    interface::{ICommand, IInteger},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, IntegerId, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct CommandNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) value: ImmOrPNode<IntegerId>,
    pub(crate) command_value: ImmOrPNode<IntegerId>,
    pub(crate) polling_time: Option<u64>,
}

impl CommandNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn value_elem(&self) -> ImmOrPNode<IntegerId> {
        self.value
    }

    #[must_use]
    pub fn command_value_elem(&self) -> ImmOrPNode<IntegerId> {
        self.command_value
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

impl ICommand for CommandNode {
    fn execute<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.elem_base.verify_is_writable(device, store, cx)?;
        cx.invalidate_cache_by(self.node_base().id());

        let value = self.command_value.value(device, store, cx)?;
        self.value.set_value(value, device, store, cx)
    }

    fn is_done<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        let nid = match self.value {
            ImmOrPNode::Imm(..) => return Ok(true),
            ImmOrPNode::PNode(nid) => nid,
        };

        cx.invalidate_cache_of(nid);
        let node = nid.expect_iinteger_kind(store)?;
        if !node.is_readable(device, store, cx)? {
            Ok(true)
        } else {
            let command_value = self.command_value.value(device, store, cx)?;
            let reg_value = node.value(device, store, cx)?;
            Ok(command_value != reg_value)
        }
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
