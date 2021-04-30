use crate::GenApiError;

use super::{
    interface::{IRegister, IString},
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
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<String> {
        let nid = self.node_base().id();
        let reg = self.register_base();
        let res = if let Some(cache) = cx.get_cached(nid) {
            let res = String::from_utf8_lossy(cache).to_string();
            // Avoid a lifetime problem.
            reg.elem_base.verify_is_readable(device, store, cx)?;
            res
        } else {
            let bytes = reg.read_then_cache(nid, device, store, cx)?;
            let len = bytes
                .iter()
                .position(|c| *c == 0)
                .unwrap_or_else(|| bytes.len());
            String::from_utf8_lossy(&bytes[0..len]).to_string()
        };
        Ok(res.into())
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        if !value.is_ascii() {
            return Err(GenApiError::InvalidData(
                "the data to write must be an ascii string".into(),
            ));
        }
        if value.len() > self.max_length(device, store, cx)? as usize {
            return Err(GenApiError::InvalidData(
                "the data to write exceeds the maximum length allowed by the node.".into(),
            ));
        }
        let nid = self.node_base().id();
        cx.invalidate_cache_by(nid);
        let reg = self.register_base();
        let mut buf = vec![0_u8; self.length(device, store, cx)? as usize];
        let bytes = value.as_bytes();
        buf[0..bytes.len()].copy_from_slice(bytes);
        reg.write_then_cache(nid, &buf, device, store, cx)
    }

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.length(device, store, cx)
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base()
            .elem_base
            .is_readable(device, store, cx)
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base()
            .elem_base
            .is_writable(device, store, cx)
    }
}

impl IRegister for StringRegNode {
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.register_base()
            .read_then_cache_with_buf(self.node_base().id(), buf, device, store, cx)
    }

    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.register_base()
            .write_then_cache(self.node_base().id(), buf, device, store, cx)
    }

    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.register_base().address(device, store, cx)
    }

    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.register_base().length(device, store, cx)
    }
}
