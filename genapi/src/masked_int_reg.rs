use super::{
    elem_type::{BitMask, Endianness, IntegerRepresentation, Sign},
    interface::{IInteger, IRegister, ISelector, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase},
    register_base::RegisterBase,
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct MaskedIntRegNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) register_base: RegisterBase,

    pub(crate) bit_mask: BitMask,
    pub(crate) sign: Sign,
    pub(crate) endianness: Endianness,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
    pub(crate) p_selected: Vec<NodeId>,
}

impl MaskedIntRegNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }

    #[must_use]
    pub fn bit_mask(&self) -> BitMask {
        self.bit_mask
    }

    #[must_use]
    pub fn sign(&self) -> Sign {
        self.sign
    }

    #[must_use]
    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    #[must_use]
    pub fn unit_elem(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation_elem(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

impl IInteger for MaskedIntRegNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let nid = self.node_base().id();
        let reg = self.register_base();

        // Get register value.
        let reg_value = reg.with_cache_or_read(nid, device, store, cx, |data| {
            utils::int_from_slice(data, self.endianness, self.sign)
        })?;

        // Apply mask.
        let len = reg.length(device, store, cx)? as usize;
        let res = self
            .bit_mask
            .apply_mask(reg_value, len, self.endianness, self.sign);

        Ok(res)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let nid = self.node_base().id();
        cx.invalidate_cache_by(nid);

        let min = self.min(device, store, cx)?;
        let max = self.max(device, store, cx)?;
        utils::verify_value_in_range(&value, &min, &max)?;

        let reg = self.register_base();
        let old_reg_value = reg.with_cache_or_read(nid, device, store, cx, |data| {
            utils::int_from_slice(data, self.endianness, self.sign)
        })?;

        let length = reg.length(device, store, cx)? as usize;
        let new_reg_value =
            self.bit_mask
                .masked_value(old_reg_value, value, length, self.endianness);
        let mut buf = vec![0; length as usize];
        utils::bytes_from_int(new_reg_value, &mut buf, self.endianness, self.sign)?;
        reg.write_and_cache(nid, &buf, device, store, cx)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let len = self.register_base().length(device, store, cx)? as usize;
        Ok(self.bit_mask.min(len, self.endianness, self.sign))
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let len = self.register_base().length(device, store, cx)? as usize;
        Ok(self.bit_mask.max(len, self.endianness, self.sign))
    }

    fn inc_mode(&self, _: &impl NodeStore) -> Option<IncrementMode> {
        None
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        Ok(None)
    }

    fn valid_value_set(&self, _: &impl NodeStore) -> &[i64] {
        &[]
    }

    fn representation(&self, _: &impl NodeStore) -> IntegerRepresentation {
        self.representation_elem()
    }

    fn unit(&self, _: &impl NodeStore) -> Option<&str> {
        self.unit_elem()
    }

    #[tracing::instrument(skip(self, store),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        _: i64,
        _: &mut impl Device,
        store: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::not_writable())
    }

    #[tracing::instrument(skip(self, store),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        _: i64,
        _: &mut impl Device,
        store: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::not_writable())
    }

    fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base().is_readable(device, store, cx)
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.register_base().is_writable(device, store, cx)
    }
}

impl IRegister for MaskedIntRegNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn read<T: ValueStore, U: CacheStore>(
        &self,
        buf: &mut [u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        let address = self.address(device, store, cx)?;
        let length = self.length(device, store, cx)?;
        self.register_base().read_and_cache(
            self.node_base().id(),
            address,
            length,
            buf,
            device,
            store,
            cx,
        )
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn write<T: ValueStore, U: CacheStore>(
        &self,
        buf: &[u8],
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        self.register_base()
            .write_and_cache(self.node_base().id(), buf, device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn address<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.register_base().address(device, store, cx)
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn length<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        self.register_base().length(device, store, cx)
    }
}

impl ISelector for MaskedIntRegNode {
    fn selecting_nodes(&self, _: &impl NodeStore) -> GenApiResult<&[NodeId]> {
        Ok(self.p_selected())
    }
}

impl BitMask {
    fn apply_mask(
        &self,
        reg_value: i64,
        reg_byte_len: usize,
        endianness: Endianness,
        sign: Sign,
    ) -> i64 {
        let mask = self.mask(reg_byte_len, endianness);
        let (lsb, msb) = (
            self.lsb(reg_byte_len, endianness),
            self.msb(reg_byte_len, endianness),
        );
        let res = (reg_value & mask) >> lsb;

        match sign {
            Sign::Signed if res >> (msb - lsb) == 1 => {
                // Do sign extension.
                res | ((-1) ^ (mask >> lsb))
            }
            _ => res,
        }
    }

    fn masked_value(
        &self,
        old_reg_value: i64,
        value: i64,
        reg_byte_len: usize,
        endianness: Endianness,
    ) -> i64 {
        let mask = self.mask(reg_byte_len, endianness);
        let lsb = self.lsb(reg_byte_len, endianness);
        (old_reg_value & !mask) | ((value << lsb) & mask)
    }

    fn lsb(self, reg_byte_len: usize, endianness: Endianness) -> usize {
        let lsb = match self {
            Self::SingleBit(lsb) | Self::Range { lsb, .. } => lsb as usize,
        };

        // Normalize the value.
        let bits_len = reg_byte_len * 8;
        match endianness {
            Endianness::LE => lsb,
            Endianness::BE => (bits_len - lsb - 1),
        }
    }

    fn msb(self, reg_byte_len: usize, endianness: Endianness) -> usize {
        let msb = match self {
            Self::SingleBit(msb) | Self::Range { msb, .. } => msb as usize,
        };

        // Normalize the value.
        let bits_len = reg_byte_len * 8;
        match endianness {
            Endianness::LE => msb,
            Endianness::BE => (bits_len - msb - 1),
        }
    }

    fn min(&self, reg_byte_len: usize, endianness: Endianness, sign: Sign) -> i64 {
        let (lsb, msb) = (
            self.lsb(reg_byte_len, endianness),
            self.msb(reg_byte_len, endianness),
        );
        match sign {
            Sign::Signed => {
                if msb - lsb == 63 {
                    std::i64::MIN
                } else {
                    let value = 1 << (msb - lsb) as i64;
                    -value
                }
            }
            Sign::Unsigned => 0,
        }
    }

    fn max(&self, reg_byte_len: usize, endianness: Endianness, sign: Sign) -> i64 {
        let (lsb, msb) = (
            self.lsb(reg_byte_len, endianness),
            self.msb(reg_byte_len, endianness),
        );
        if msb - lsb == 63 {
            return std::i64::MAX;
        }
        match sign {
            Sign::Signed => (1 << (msb - lsb)) - 1,
            Sign::Unsigned => {
                if msb - lsb == 63 {
                    std::i64::MAX
                } else {
                    (1 << (msb - lsb + 1)) - 1
                }
            }
        }
    }

    fn mask(&self, reg_byte_len: usize, endianness: Endianness) -> i64 {
        let (lsb, msb) = (
            self.lsb(reg_byte_len, endianness),
            self.msb(reg_byte_len, endianness),
        );
        if msb - lsb == 63 {
            -1
        } else {
            ((1 << (msb - lsb + 1)) - 1) << lsb
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::shadow_unrelated)]
    use super::*;

    #[test]
    fn test_bit_mask_8bit_single_bit() {
        let reg_len = 1;
        let reg_value = 0b1100_1011;
        let endianness = Endianness::LE;
        let mask = BitMask::SingleBit(3);

        let sign = Sign::Unsigned;
        assert_eq!(mask.min(reg_len, endianness, sign), 0);
        assert_eq!(mask.max(reg_len, endianness, sign), 1);
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, 1);
        let new_value = mask.masked_value(reg_value, 0, reg_len, endianness);
        assert_eq!(new_value, 0b1100_0011);

        let sign = Sign::Signed;
        assert_eq!(mask.min(reg_len, endianness, sign), -1);
        assert_eq!(mask.max(reg_len, endianness, sign), 0);
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, -1);
        let new_value = mask.masked_value(reg_value, 0, reg_len, endianness);
        assert_eq!(new_value, 0b1100_0011);
    }

    #[test]
    fn test_bit_mask_8bit_le() {
        let reg_len = 1;
        let reg_value = 0b1100_1011;
        let endianness = Endianness::LE;
        let mask = BitMask::Range { lsb: 1, msb: 4 };

        let sign = Sign::Unsigned;
        assert_eq!(mask.min(reg_len, endianness, sign), 0);
        assert_eq!(mask.max(reg_len, endianness, sign), 15);
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, 0b0101);
        let new_value = mask.masked_value(reg_value, 0b0110, reg_len, endianness);
        assert_eq!(new_value, 0b1100_1101);

        let sign = Sign::Signed;
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(mask.min(reg_len, endianness, sign), -8);
        assert_eq!(mask.max(reg_len, endianness, sign), 7);
        assert_eq!(value, 5);
        let new_value = mask.masked_value(reg_value, -1, reg_len, endianness);
        assert_eq!(new_value, 0b1101_1111);
    }

    #[test]
    fn test_bit_mask_8bit_be() {
        let reg_len = 1;
        let reg_value = 0b1100_1011;
        let endianness = Endianness::BE;
        let mask = BitMask::Range { lsb: 6, msb: 3 };

        let sign = Sign::Unsigned;
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, 0b0101);
        let new_value = mask.masked_value(reg_value, 0b0110, reg_len, endianness);
        assert_eq!(new_value, 0b1100_1101);

        let sign = Sign::Signed;
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, 5);
        let new_value = mask.masked_value(reg_value, -1, reg_len, endianness);
        assert_eq!(new_value, 0b1101_1111);
    }

    #[test]
    fn test_bit_mask_64bit() {
        let reg_len = 1;
        let reg_value = i64::MAX;
        let endianness = Endianness::LE;
        let mask = BitMask::Range { lsb: 0, msb: 63 };

        let sign = Sign::Unsigned;
        assert_eq!(mask.min(reg_len, endianness, sign), 0);
        assert_eq!(mask.max(reg_len, endianness, sign), std::i64::MAX);
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, i64::MAX);
        let new_value = mask.masked_value(reg_value, 0, reg_len, endianness);
        assert_eq!(new_value, 0);

        let sign = Sign::Signed;
        assert_eq!(mask.min(reg_len, endianness, sign), std::i64::MIN);
        assert_eq!(mask.max(reg_len, endianness, sign), std::i64::MAX);
        let value = mask.apply_mask(reg_value, reg_len, endianness, sign);
        assert_eq!(value, std::i64::MAX);
        let new_value = mask.masked_value(reg_value, std::i64::MIN, reg_len, endianness);
        assert_eq!(new_value, std::i64::MIN);
    }
}
