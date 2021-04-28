use super::{
    elem_type::{DisplayNotation, Endianness, FloatRepresentation},
    interface::{IFloat, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase},
    store::{CacheStore, NodeStore, ValueStore},
    Device, GenApiResult, RegisterBase, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct FloatRegNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) register_base: RegisterBase,

    pub(crate) endianness: Endianness,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
}

impl FloatRegNode {
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
    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    #[must_use]
    pub fn unit_elem(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation_elem(&self) -> FloatRepresentation {
        self.representation
    }

    #[must_use]
    pub fn display_notation_elem(&self) -> DisplayNotation {
        self.display_notation
    }

    #[must_use]
    pub fn display_precision_elem(&self) -> i64 {
        self.display_precision
    }
}

impl IFloat for FloatRegNode {
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        todo!()
    }

    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo! {}
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        todo!()
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        todo!()
    }

    fn inc_mode(&self, store: impl NodeStore) -> GenApiResult<Option<IncrementMode>> {
        todo!()
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        todo!()
    }

    /// NOTE: `ValidValueSet` is not supported in `GenApiSchema Version 1.1` yet.
    fn valid_value_set(&self, store: impl NodeStore) -> &[f64] {
        todo!()
    }

    fn representation(&self, store: impl NodeStore) -> FloatRepresentation {
        todo!()
    }

    fn unit(&self, store: impl NodeStore) -> Option<&str> {
        todo!()
    }

    fn display_notation(&self, store: impl NodeStore) -> DisplayNotation {
        todo!()
    }

    fn display_precision(&self, store: impl NodeStore) -> i64 {
        todo! {}
    }

    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }

    fn set_max<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
        device: impl Device,
        store: impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        todo!()
    }
}
