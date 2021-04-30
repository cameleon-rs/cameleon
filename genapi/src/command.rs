use super::{
    elem_type::ImmOrPNode,
    interface::ICommand,
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
        todo!()
    }

    fn is_done<T: ValueStore, U: CacheStore>(
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
