use super::{
    elem_type::ImmOrPNode,
    interface::IPort,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct PortNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) chunk_id: Option<ImmOrPNode<u64>>,
    pub(crate) swap_endianness: bool,
    pub(crate) cache_chunk_data: bool,
}

impl PortNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn chunk_id(&self) -> Option<&ImmOrPNode<u64>> {
        self.chunk_id.as_ref()
    }

    #[must_use]
    pub fn swap_endianness(&self) -> bool {
        self.swap_endianness
    }

    #[must_use]
    pub fn cache_chunk_data(&self) -> bool {
        self.cache_chunk_data
    }
}

impl IPort for PortNode {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &mut [u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &[u8],
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }
}
