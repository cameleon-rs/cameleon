use super::{
    elem_type::{DisplayNotation, FloatRepresentation, IntegerRepresentation},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    EnumEntryNode, {Context, Device, GenApiResult},
};

#[derive(Clone, Debug)]
pub enum IncrementMode {
    FixedIncrement,
    /// NOTE: `ListIncrement` is not supported in `GenApiSchema Version 1.1` yet.
    ListIncrement,
}

pub trait IInteger {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<i64>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<i64>;

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<i64>;

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>>;

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<Option<i64>>;

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: impl NodeStore) -> &[i64];

    fn representation(&self, store: impl NodeStore) -> IntegerRepresentation;

    fn unit(&self, store: impl NodeStore) -> Option<&str>;

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: i64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn selected_by(&self, store: impl NodeStore) -> &[NodeId];
}

pub trait IFloat {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<f64>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<f64>;

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<f64>;

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>>;

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<Option<f64>>;

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: impl NodeStore) -> &[f64];

    fn representation(&self, store: impl NodeStore) -> FloatRepresentation;

    fn unit(&self, store: impl NodeStore) -> Option<&str>;

    fn display_notation(&self, store: impl NodeStore) -> DisplayNotation;

    fn display_precision(&self, store: impl NodeStore) -> i64;

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;
}

pub trait IString {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<String>;

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: &str,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn max_length<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<usize>;
}

pub trait IEnumeration {
    fn current_entry<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<&EnumEntryNode>;

    fn entries(&self, store: impl NodeStore) -> GenApiResult<&[EnumEntryNode]>;

    fn set_entry_by_name<T: ValueStore, U: CacheStore>(
        &self,
        name: &str,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;

    fn set_entry_by_idx<T: ValueStore, U: CacheStore>(
        &self,
        idx: usize,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut Context<T, U>,
    ) -> GenApiResult<()>;
}
