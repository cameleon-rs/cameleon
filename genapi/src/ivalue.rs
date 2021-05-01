use super::{
    elem_type::{ImmOrPNode, PIndex, PValue, ValueKind},
    interface::{IFloat, IInteger},
    store::{CacheStore, FloatId, IntegerId, NodeId, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
};

pub trait IValue<T> {
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
}

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
}

impl IValue<i64> for IntegerId {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<i64> {
        Ok(cx.value_store.integer_value(*self).unwrap())
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: i64,
        _: &mut impl Device,
        _: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        cx.value_store_mut().update(*self, value);
        Ok(())
    }
}

impl IValue<i64> for i64 {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<i64> {
        Ok(*self)
    }

    fn set_value<U, S>(
        &self,
        _: i64,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        Err(GenApiError::access_denied(
            "cannot rewrite the constant".into(),
        ))
    }
}

impl IValue<i64> for NodeId {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<i64> {
        self.expect_iinteger_kind(store)?.value(device, store, cx)
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: i64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        self.expect_iinteger_kind(store)?
            .set_value(value, device, store, cx)
    }
}

impl IValue<f64> for FloatId {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<f64> {
        Ok(cx.value_store().float_value(*self).unwrap())
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: f64,
        _: &mut impl Device,
        _: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        cx.value_store_mut().update(*self, value);
        Ok(())
    }
}

impl IValue<f64> for f64 {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<f64> {
        Ok(*self)
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        _: f64,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        Err(GenApiError::access_denied(
            "cannot rewrite the constant".into(),
        ))
    }
}

impl IValue<f64> for NodeId {
    fn value<U: ValueStore, S: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<f64> {
        self.expect_ifloat_kind(store)?.value(device, store, cx)
    }

    fn set_value<U: ValueStore, S: CacheStore>(
        &self,
        value: f64,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<U, S>,
    ) -> GenApiResult<()> {
        self.expect_ifloat_kind(store)?
            .set_value(value, device, store, cx)
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
        let index = self
            .p_index
            .expect_iinteger_kind(store)?
            .value(device, store, cx)?;
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
        let index = self
            .p_index
            .expect_iinteger_kind(store)?
            .value(device, store, cx)?;
        if let Some(value_indexed) = self.value_indexed.iter().find(|vi| vi.index == index) {
            value_indexed.indexed.set_value(value, device, store, cx)
        } else {
            self.value_default.set_value(value, device, store, cx)
        }
    }
}
