/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    elem_type::{AccessMode, AddressKind, CachingMode, ImmOrPNode},
    interface::IPort,
    ivalue::IValue,
    node_base::NodeElementBase,
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct RegisterBase {
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) address_kinds: Vec<AddressKind>,
    pub(crate) length: ImmOrPNode<i64>,
    pub(crate) access_mode: AccessMode,
    pub(crate) p_port: NodeId,
    pub(crate) cacheable: CachingMode,
    pub(crate) polling_time: Option<u64>,
    pub(crate) p_invalidators: Vec<NodeId>,
}

impl RegisterBase {
    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn address_kinds(&self) -> &[AddressKind] {
        &self.address_kinds
    }

    #[must_use]
    pub fn length_elem(&self) -> &ImmOrPNode<i64> {
        &self.length
    }

    #[must_use]
    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    #[must_use]
    pub fn p_port(&self) -> NodeId {
        self.p_port
    }

    #[must_use]
    pub fn cacheable(&self) -> CachingMode {
        self.cacheable
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    pub(super) fn with_cache_or_read<T: ValueStore, U: CacheStore, R>(
        &self,
        nid: NodeId,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
        f: impl FnOnce(&[u8]) -> GenApiResult<R>,
    ) -> GenApiResult<R> {
        let length = self.length(device, store, cx)?;
        let address = self.address(device, store, cx)?;
        if let Some(cache) = cx.get_cache(nid, length, address) {
            f(cache)
        } else {
            let mut buf = vec![0; length as usize];
            self.read_and_cache(nid, address, length, &mut buf, device, store, cx)?;
            f(&buf)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn read_and_cache<T: ValueStore, U: CacheStore>(
        &self,
        nid: NodeId,
        address: i64,
        length: i64,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        if buf.len() != length as usize {
            return Err(GenApiError::invalid_buffer(
                "given buffer length doesn't same as the register length".into(),
            ));
        }
        self.p_port
            .expect_iport_kind(store)?
            .read(address, buf, device, store, cx)?;
        if self.cacheable != CachingMode::NoCache {
            cx.cache_data(nid, address, length, &buf);
        }

        Ok(())
    }

    pub(super) fn write_and_cache<T: ValueStore, U: CacheStore>(
        &self,
        nid: NodeId,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let length = self.length(device, store, cx)?;

        if buf.len() != length as usize {
            return Err(GenApiError::invalid_buffer(
                "given buffer length doesn't same as the register length".into(),
            ));
        }

        let address = self.address(device, store, cx)?;
        self.p_port
            .expect_iport_kind(store)?
            .write(address, buf, device, store, cx)?;

        if self.cacheable == CachingMode::WriteThrough {
            cx.cache_data(nid, address, length, &buf);
        }
        Ok(())
    }

    pub(super) fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let mut address = 0;
        for addr_kind in self.address_kinds() {
            address += addr_kind.value(device, store, cx)?;
        }
        Ok(address)
    }

    pub(super) fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.length_elem().value(device, store, cx)
    }

    pub(super) fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        Ok(self.elem_base.is_readable(device, store, cx)?
            && !matches!(self.access_mode(), AccessMode::WO))
    }

    pub(super) fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        Ok(self.elem_base.is_writable(device, store, cx)?
            && !matches!(self.access_mode(), AccessMode::RO))
    }
}
