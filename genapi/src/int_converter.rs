/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    elem_type::{IntegerRepresentation, NamedValue, Slope},
    formula::{EvaluationResult, Expr, Formula},
    interface::{IInteger, INode, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, ValueCtxt,
};

fn expr_as_integer(expr: Expr) -> i64 {
    match expr {
        Expr::Integer(i) => i,
        Expr::Float(f) => f as i64,
        _ => unreachable!("node min/max/inc values must evaluate to immediate expressions"),
    }
}

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

    fn eval_formula_from<T: ValueStore, U: CacheStore>(
        &self,
        to: impl Into<Expr>,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<EvaluationResult> {
        let mut collector =
            utils::FormulaEnvCollector::new(&self.p_variables, &self.constants, &self.expressions);
        collector.insert_imm("TO", to);
        let var_env = collector.collect(device, store, cx)?;

        self.formula_from.eval(&var_env)
    }
}

impl INode for IntConverterNode {
    fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        self.streamable
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
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let raw_min = utils::min_from_nid(self.p_value, device, store, cx)?;
        self.eval_formula_from(raw_min, device, store, cx)
            .map(|value| value.as_integer())
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<i64> {
        let raw_max = utils::max_from_nid(self.p_value, device, store, cx)?;
        self.eval_formula_from(raw_max, device, store, cx)
            .map(|value| value.as_integer())
    }

    fn inc_mode(&self, _: &impl NodeStore) -> Option<IncrementMode> {
        None
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<i64>> {
        let Some(raw_inc) = utils::inc_from_nid(self.p_value, device, store, cx)? else {
            return Ok(None);
        };
        let raw_min = expr_as_integer(utils::min_from_nid(self.p_value, device, store, cx)?);
        let raw_inc = expr_as_integer(raw_inc);
        let min_plus_inc = self
            .eval_formula_from(raw_min + raw_inc, device, store, cx)?
            .as_integer();
        let min = self
            .eval_formula_from(raw_min, device, store, cx)?
            .as_integer();

        Ok(Some(min_plus_inc - min))
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
