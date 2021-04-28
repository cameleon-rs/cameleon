use super::{
    interface::IString,
    node_base::{NodeAttributeBase, NodeBase},
    register_base::RegisterBase,
    store::{CacheStore, NodeStore, ValueStore},
    Device, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct StringRegNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) register_base: RegisterBase,
}

impl StringRegNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }
}

impl IString for StringRegNode {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String> {
        todo!()
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        todo!()
    }
}
