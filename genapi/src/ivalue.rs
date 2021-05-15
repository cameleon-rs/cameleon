use ambassador::delegatable_trait;

use super::{
    elem_type::{ImmOrPNode, PIndex, PValue, ValueKind},
    interface::{IFloat, IInteger, IString},
    store::{CacheStore, FloatId, IntegerId, NodeId, NodeStore, StringId, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
};

#[delegatable_trait]
pub(super) trait IValue<T> {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<T>;

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: T,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()>;

    fn is_readable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool>;

    fn is_writable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool>;
}

macro_rules! impl_ivalue_for_imm {
    ($ty:ty) => {
        impl IValue<$ty> for $ty {
            fn value<U: ValueStore, S: CacheStore>(
                &self,
                _: &mut impl Device,
                _: &impl NodeStore,
                _: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<$ty> {
                Ok(*self)
            }

            fn set_value<U, S>(
                &self,
                _: $ty,
                _: &mut impl Device,
                _: &impl NodeStore,
                _: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<()> {
                Err(GenApiError::not_writable())
            }

            fn is_readable<U: ValueStore, S: CacheStore>(
                &self,
                _: &mut impl Device,
                _: &impl NodeStore,
                _: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<bool> {
                Ok(true)
            }

            fn is_writable<U: ValueStore, S: CacheStore>(
                &self,
                _: &mut impl Device,
                _: &impl NodeStore,
                _: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<bool> {
                Ok(false)
            }
        }
    };
}
impl_ivalue_for_imm!(i64);
impl_ivalue_for_imm!(f64);

macro_rules! impl_ivalue_for_vid {
    ($ty:ty, $vid:ty, $f:ident) => {
        impl IValue<$ty> for $vid {
            fn value<U: ValueStore, S: CacheStore>(
                &self,
                _: &mut impl Device,
                _: &impl NodeStore,
                cx: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<$ty> {
                Ok(cx.value_store.$f(*self).unwrap().into())
            }

            fn set_value<U: ValueStore, S: CacheStore>(
                &self,
                value: $ty,
                _: &mut impl Device,
                _: &impl NodeStore,
                cx: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<()> {
                cx.value_store_mut().update(*self, value);
                Ok(())
            }

            fn is_readable<U: ValueStore, S: CacheStore>(
                &self,
                _: &mut impl Device,
                _: &impl NodeStore,
                _: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<bool> {
                Ok(true)
            }

            fn is_writable<U: ValueStore, S: CacheStore>(
                &self,
                _: &mut impl Device,
                _: &impl NodeStore,
                _: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<bool> {
                Ok(true)
            }
        }
    };
}
impl_ivalue_for_vid!(i64, IntegerId, integer_value);
impl_ivalue_for_vid!(f64, FloatId, float_value);
impl_ivalue_for_vid!(String, StringId, str_value);

macro_rules! impl_ivalue_for_node_id {
    ($ty:ty, $expect_kind:ident) => {
        impl IValue<$ty> for NodeId {
            fn value<U: ValueStore, S: CacheStore>(
                &self,
                device: &mut impl Device,
                store: &impl NodeStore,
                cx: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<$ty> {
                self.$expect_kind(store)?.value(device, store, cx)
            }

            fn set_value<U: ValueStore, S: CacheStore>(
                &self,
                value: $ty,
                device: &mut impl Device,
                store: &impl NodeStore,
                cx: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<()> {
                self.$expect_kind(store)?
                    .set_value(value, device, store, cx)
            }

            fn is_readable<U: ValueStore, S: CacheStore>(
                &self,
                device: &mut impl Device,
                store: &impl NodeStore,
                cx: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<bool> {
                self.$expect_kind(store)?.is_readable(device, store, cx)
            }

            fn is_writable<U: ValueStore, S: CacheStore>(
                &self,
                device: &mut impl Device,
                store: &impl NodeStore,
                cx: &mut ValueCtxt<U, S>,
            ) -> GenApiResult<bool> {
                self.$expect_kind(store)?.is_writable(device, store, cx)
            }
        }
    };
}
impl_ivalue_for_node_id!(f64, expect_ifloat_kind);
impl_ivalue_for_node_id!(i64, expect_iinteger_kind);
impl_ivalue_for_node_id!(String, expect_istring_kind);

impl<T, Ty> IValue<T> for ImmOrPNode<Ty>
where
    Ty: IValue<T>,
    NodeId: IValue<T>,
{
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<T> {
        match self {
            ImmOrPNode::Imm(i) => i.value(device, store, cx),
            ImmOrPNode::PNode(nid) => nid.value(device, store, cx),
        }
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: T,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        match self {
            ImmOrPNode::Imm(i) => i.set_value(value, device, store, cx),
            ImmOrPNode::PNode(nid) => nid.set_value(value, device, store, cx),
        }
    }

    fn is_readable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        match self {
            ImmOrPNode::Imm(i) => i.is_readable(device, store, cx),
            ImmOrPNode::PNode(nid) => nid.is_readable(device, store, cx),
        }
    }

    fn is_writable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        match self {
            ImmOrPNode::Imm(i) => i.is_writable(device, store, cx),
            ImmOrPNode::PNode(nid) => nid.is_writable(device, store, cx),
        }
    }
}

impl<T, Ty> IValue<T> for ValueKind<Ty>
where
    Ty: IValue<T>,
    PValue<Ty>: IValue<T>,
    PIndex<Ty>: IValue<T>,
{
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<T> {
        match self {
            ValueKind::Value(i) => i.value(device, store, cx),
            ValueKind::PValue(p_value) => p_value.value(device, store, cx),
            ValueKind::PIndex(p_index) => p_index.value(device, store, cx),
        }
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: T,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        match self {
            ValueKind::Value(i) => i.set_value(value, device, store, cx),
            ValueKind::PValue(p_value) => p_value.set_value(value, device, store, cx),
            ValueKind::PIndex(p_index) => p_index.set_value(value, device, store, cx),
        }
    }

    fn is_readable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        match self {
            ValueKind::Value(i) => i.is_readable(device, store, cx),
            ValueKind::PValue(p_value) => p_value.is_readable(device, store, cx),
            ValueKind::PIndex(p_index) => p_index.is_readable(device, store, cx),
        }
    }

    fn is_writable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        match self {
            ValueKind::Value(i) => i.is_writable(device, store, cx),
            ValueKind::PValue(p_value) => p_value.is_writable(device, store, cx),
            ValueKind::PIndex(p_index) => p_index.is_writable(device, store, cx),
        }
    }
}

impl<T, Ty> IValue<T> for PValue<Ty>
where
    NodeId: IValue<T>,
    T: Copy,
{
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<T> {
        self.p_value.value(device, store, cx)
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: T,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        self.p_value.set_value(value, device, store, cx)?;
        for nid in self.p_value_copies() {
            nid.set_value(value, device, store, cx)?;
        }
        Ok(())
    }

    fn is_readable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        self.p_value.is_readable(device, store, cx)
    }

    fn is_writable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        let mut b = self.p_value.is_writable(device, store, cx)?;
        for nid in self.p_value_copies() {
            b &= nid.is_writable(device, store, cx)?;
        }
        Ok(b)
    }
}

impl<T, Ty> IValue<T> for PIndex<Ty>
where
    Ty: IValue<T>,
    ImmOrPNode<Ty>: IValue<T>,
{
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<T> {
        let index = self.index(device, store, cx)?;
        if let Some(value_indexed) = self.value_indexed.iter().find(|vi| vi.index == index) {
            value_indexed.indexed.value(device, store, cx)
        } else {
            self.value_default.value(device, store, cx)
        }
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: T,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        let index = self.index(device, store, cx)?;
        if let Some(value_indexed) = self.value_indexed.iter().find(|vi| vi.index == index) {
            value_indexed.indexed.set_value(value, device, store, cx)
        } else {
            self.value_default.set_value(value, device, store, cx)
        }
    }

    fn is_readable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        Ok(self
            .p_index
            .expect_iinteger_kind(store)?
            .is_readable(device, store, cx)?
            && {
                let index = self.index(device, store, cx)?;
                if let Some(value_indexed) = self.value_indexed.iter().find(|vi| vi.index == index)
                {
                    value_indexed.indexed.is_readable(device, store, cx)?
                } else {
                    self.value_default.is_readable(device, store, cx)?
                }
            })
    }

    fn is_writable<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<bool> {
        Ok(self
            .p_index
            .expect_iinteger_kind(store)?
            .is_readable(device, store, cx)?
            && {
                let index = self.index(device, store, cx)?;
                if let Some(value_indexed) = self.value_indexed.iter().find(|vi| vi.index == index)
                {
                    value_indexed.indexed.is_writable(device, store, cx)?
                } else {
                    self.value_default.is_writable(device, store, cx)?
                }
            })
    }
}

impl<T> PIndex<T> {
    fn index<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<i64> {
        self.p_index
            .expect_iinteger_kind(store)?
            .value(device, store, cx)
    }
}
