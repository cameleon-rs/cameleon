use super::{
    elem_type::{DisplayNotation, FloatRepresentation, NamedValue, Slope},
    formula::{Expr, Formula},
    interface::{IFloat, INode, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, ValueCtxt,
};

#[derive(Debug, Clone)]
pub struct ConverterNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) streamable: bool,
    pub(crate) p_variables: Vec<NamedValue<NodeId>>,
    pub(crate) constants: Vec<NamedValue<f64>>,
    pub(crate) expressions: Vec<NamedValue<Expr>>,
    pub(crate) formula_to: Formula,
    pub(crate) formula_from: Formula,
    pub(crate) p_value: NodeId,
    pub(crate) unit: Option<String>,
    pub(crate) representation: FloatRepresentation,
    pub(crate) display_notation: DisplayNotation,
    pub(crate) display_precision: i64,
    pub(crate) slope: Slope,
    pub(crate) is_linear: bool,
}

impl ConverterNode {
    #[must_use]
    pub fn p_variables(&self) -> &[NamedValue<NodeId>] {
        &self.p_variables
    }

    #[must_use]
    pub fn constants(&self) -> &[NamedValue<f64>] {
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

    #[must_use]
    pub fn slope(&self) -> Slope {
        self.slope
    }

    #[must_use]
    pub fn is_linear(&self) -> bool {
        self.is_linear
    }
}

impl INode for ConverterNode {
    fn node_base(&self) -> NodeBase {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        self.streamable
    }
}

impl IFloat for ConverterNode {
    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn value<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        let mut collector =
            utils::FormulaEnvCollector::new(&self.p_variables, &self.constants, &self.expressions);
        collector.insert("TO", self.p_value(), device, store, cx)?;
        let var_env = collector.collect(device, store, cx)?;

        let eval_result = self.formula_from.eval(&var_env)?;
        Ok(eval_result.as_float())
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_value<T: ValueStore, U: CacheStore>(
        &self,
        value: f64,
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
    ) -> GenApiResult<f64> {
        Ok(f64::MIN)
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        Ok(f64::MAX)
    }

    fn inc_mode(&self, _: &impl NodeStore) -> Option<IncrementMode> {
        None
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        _: &mut impl Device,
        _: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        Ok(None)
    }

    fn representation(&self, _: &impl NodeStore) -> FloatRepresentation {
        self.representation
    }

    fn unit(&self, _: &impl NodeStore) -> Option<&str> {
        self.unit_elem()
    }

    fn display_notation(&self, _: &impl NodeStore) -> DisplayNotation {
        self.display_notation
    }

    fn display_precision(&self, _: &impl NodeStore) -> i64 {
        self.display_precision
    }

    #[tracing::instrument(skip(self, store),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
    fn set_min<T: ValueStore, U: CacheStore>(
        &self,
        _: f64,
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
        _: f64,
        _: &mut impl Device,
        store: &impl NodeStore,
        _: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        Err(GenApiError::not_writable())
    }

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
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

    #[tracing::instrument(skip(self, device, store, cx),
                          level = "trace",
                          fields(node = store.name_by_id(self.node_base().id()).unwrap()))]
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
