use super::{
    elem_type::ImmOrPNode,
    interface::{INode, IPort},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
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

impl INode for PortNode {
    fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        false
    }
}

impl IPort for PortNode {
    #[tracing::instrument(skip(self, device, store),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        if self.chunk_id.is_some() {
            Err(GenApiError::chunk_data_missing())
        } else {
            device
                .read_mem(address, buf)
                .map_err(|e| GenApiError::device(Box::new(e)))
        }
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn write<T: ValueStore, U: CacheStore>(
        &self,
        address: i64,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        cx.invalidate_cache_by(self.node_base().id());

        if self.chunk_id.is_some() {
            // TODO: Implement chunk parser.
            todo!()
        } else {
            device
                .write_mem(address, buf)
                .map_err(|e| GenApiError::device(Box::new(e)))
        }
    }
}
