use super::{
    elem_type::{IntegerRepresentation, NamedValue, Slope},
    formula::{Expr, Formula},
    interface::{IInteger, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct IntConverterNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) p_variables: Vec<NamedValue<NodeId>>,
    pub(crate) constants: Vec<NamedValue<i64>>,
    pub(crate) expressions: Vec<NamedValue<Expr>>,
    pub(crate) formula_to: Formula,
    pub(crate) formula_from: Formula,
    pub(crate) p_value: NodeId,
    pub(crate) unit: Option<String>,
    pub(crate) representation: IntegerRepresentation,
    pub(crate) slope: Slope,
}

impl IntConverterNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn p_variables(&self) -> &[NamedValue<NodeId>] {
        &self.p_variables
    }

    #[must_use]
    pub fn constants(&self) -> &[NamedValue<i64>] {
        &self.constants
    }

    #[must_use]
    pub fn expressions(&self) -> &[NamedValue<Expr>] {
        &self.expressions
    }

    #[must_use]
    pub fn formula_to(&self) -> &Formula {
        &self.formula_to
    }

    #[must_use]
    pub fn formula_from(&self) -> &Formula {
        &self.formula_from
    }

    #[must_use]
    pub fn p_value(&self) -> NodeId {
        self.p_value
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
    pub fn slope(&self) -> Slope {
        self.slope
    }
}

impl IInteger for IntConverterNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let mut collector =
            utils::FormulaEnvCollector::new(&self.p_variables, &self.constants, &self.expressions);
        collector.insert("TO", self.p_value(), device, store, cx)?;
        let var_env = collector.collect(device, store, cx)?;

        let eval_result = self.formula_from.eval(&var_env)?;
        Ok(eval_result.as_integer())
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
        cx.invalidate_cache_by(self.node_base().id());

        let mut collector =
            utils::FormulaEnvCollector::new(&self.p_variables, &self.constants, &self.expressions);
        collector.insert_imm("FROM", value);
        let var_env = collector.collect(device, store, cx)?;

        let eval_result = self.formula_to.eval(&var_env)?;
        utils::set_eval_result(self.p_value, eval_result, device, store, cx)?;
        Ok(())
    }

    fn min<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        Ok(i64::MIN)
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        Ok(i64::MAX)
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
        self.representation
    }

    fn unit(&self, _: &impl NodeStore) -> Option<&str> {
        None
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
        let collector =
            utils::FormulaEnvCollector::new(&self.p_variables, &self.constants, &self.expressions);
        Ok(self.elem_base.is_readable(device, store, cx)?
            && utils::is_nid_readable(self.p_value, device, store, cx)?
            && collector.is_readable(device, store, cx)?)
    }

    fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        let collector =
            utils::FormulaEnvCollector::new(&self.p_variables, &self.constants, &self.expressions);
        Ok(self.elem_base.is_writable(device, store, cx)?
            && utils::is_nid_writable(self.p_value, device, store, cx)?
            && collector.is_readable(device, store, cx)?) // Collector is needed to be readable to write a value.
    }
}
