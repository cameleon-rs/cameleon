/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    elem_type::{DisplayNotation, FloatRepresentation, NamedValue, Slope},
    formula::{EvaluationResult, Expr, Formula},
    interface::{IFloat, INode, IncrementMode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils, Device, GenApiError, GenApiResult, ValueCtxt,
};

fn expr_as_float(expr: Expr) -> f64 {
    match expr {
        Expr::Integer(i) => i as f64,
        Expr::Float(f) => f,
        _ => unreachable!("node min/max/inc values must evaluate to immediate expressions"),
    }
}

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

impl INode for ConverterNode {
    fn node_base(&self) -> NodeBase<'_> {
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
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        let raw_min = utils::min_from_nid(self.p_value, device, store, cx)?;
        self.eval_formula_from(raw_min, device, store, cx)
            .map(|value| value.as_float())
    }

    fn max<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<f64> {
        let raw_max = utils::max_from_nid(self.p_value, device, store, cx)?;
        self.eval_formula_from(raw_max, device, store, cx)
            .map(|value| value.as_float())
    }

    fn inc_mode(&self, _: &impl NodeStore) -> Option<IncrementMode> {
        None
    }

    fn inc<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<Option<f64>> {
        let Some(raw_inc) = utils::inc_from_nid(self.p_value, device, store, cx)? else {
            return Ok(None);
        };
        let raw_min = expr_as_float(utils::min_from_nid(self.p_value, device, store, cx)?);
        let raw_inc = expr_as_float(raw_inc);
        let min_plus_inc = self
            .eval_formula_from(raw_min + raw_inc, device, store, cx)?
            .as_float();
        let min = self
            .eval_formula_from(raw_min, device, store, cx)?
            .as_float();

        Ok(Some(min_plus_inc - min))
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

#[cfg(test)]
mod tests {
    use crate::{
        builder::GenApiBuilder,
        prelude::{IFloat, IInteger},
        store::{DefaultCacheStore, DefaultNodeStore, DefaultValueStore, NodeStore},
        Device, RegisterDescription, ValueCtxt,
    };

    struct DummyDevice;

    impl Device for DummyDevice {
        fn read_mem(
            &mut self,
            _: i64,
            _: &mut [u8],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            unreachable!()
        }

        fn write_mem(
            &mut self,
            _: i64,
            _: &[u8],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            unreachable!()
        }
    }

    #[test]
    fn test_converter_min_max_inc_from_p_value() {
        let xml = r#"
        <RegisterDescription
          ModelName="CameleonModel"
          VendorName="CameleonVendor"
          StandardNameSpace="None"
          SchemaMajorVersion="1"
          SchemaMinorVersion="1"
          SchemaSubMinorVersion="0"
          MajorVersion="1"
          MinorVersion="0"
          SubMinorVersion="0"
          ProductGuid="01234567-0123-0123-0123-0123456789ab"
          VersionGuid="76543210-3210-3210-3210-ba9876543210"
          xmlns="http://www.genicam.org/GenApi/Version_1_0">

            <Category Name="Root" NameSpace="Standard">
                <pFeature>Converted</pFeature>
            </Category>

            <Integer Name="Scale">
                <Value>3</Value>
            </Integer>

            <Integer Name="Raw">
                <Value>10</Value>
                <Min>10</Min>
                <Max>20</Max>
                <Inc>2</Inc>
            </Integer>

            <Float Name="Alias">
                <pValue>Raw</pValue>
            </Float>

            <Converter Name="Converted">
                <pVariable Name="Scale">Scale</pVariable>
                <FormulaTo>(FROM - 5) / Scale</FormulaTo>
                <FormulaFrom>TO * Scale + 5</FormulaFrom>
                <pValue>Alias</pValue>
             </Converter>
        </RegisterDescription>
        "#;

        let (_, node_store, mut value_ctxt): (
            RegisterDescription,
            DefaultNodeStore,
            ValueCtxt<DefaultValueStore, DefaultCacheStore>,
        ) = GenApiBuilder::<DefaultNodeStore, DefaultValueStore, DefaultCacheStore>::default()
            .build(&xml)
            .unwrap();
        let node_id = node_store.id_by_name("Converted").unwrap();
        let node = node_id.expect_ifloat_kind(&node_store).unwrap();
        let mut device = DummyDevice;

        assert_eq!(
            node.min(&mut device, &node_store, &mut value_ctxt).unwrap(),
            35.0
        );
        assert_eq!(
            node.max(&mut device, &node_store, &mut value_ctxt).unwrap(),
            65.0
        );
        assert_eq!(
            node.inc(&mut device, &node_store, &mut value_ctxt).unwrap(),
            Some(6.0)
        );
    }

    #[test]
    fn test_int_converter_min_max_inc_from_p_value() {
        let xml = r#"
        <RegisterDescription
          ModelName="CameleonModel"
          VendorName="CameleonVendor"
          StandardNameSpace="None"
          SchemaMajorVersion="1"
          SchemaMinorVersion="1"
          SchemaSubMinorVersion="0"
          MajorVersion="1"
          MinorVersion="0"
          SubMinorVersion="0"
          ProductGuid="01234567-0123-0123-0123-0123456789ab"
          VersionGuid="76543210-3210-3210-3210-ba9876543210"
          xmlns="http://www.genicam.org/GenApi/Version_1_0">

            <Category Name="Root" NameSpace="Standard">
                <pFeature>Converted</pFeature>
            </Category>

            <Integer Name="Raw">
                <Value>10</Value>
                <Min>10</Min>
                <Max>20</Max>
                <Inc>2</Inc>
            </Integer>

            <IntConverter Name="Converted">
                <FormulaTo>(FROM - 5) / 3</FormulaTo>
                <FormulaFrom>TO * 3 + 5</FormulaFrom>
                <pValue>Raw</pValue>
             </IntConverter>
        </RegisterDescription>
        "#;

        let (_, node_store, mut value_ctxt): (
            RegisterDescription,
            DefaultNodeStore,
            ValueCtxt<DefaultValueStore, DefaultCacheStore>,
        ) = GenApiBuilder::<DefaultNodeStore, DefaultValueStore, DefaultCacheStore>::default()
            .build(&xml)
            .unwrap();
        let node_id = node_store.id_by_name("Converted").unwrap();
        let node = node_id.expect_iinteger_kind(&node_store).unwrap();
        let mut device = DummyDevice;

        assert_eq!(
            node.min(&mut device, &node_store, &mut value_ctxt).unwrap(),
            35
        );
        assert_eq!(
            node.max(&mut device, &node_store, &mut value_ctxt).unwrap(),
            65
        );
        assert_eq!(
            node.inc(&mut device, &node_store, &mut value_ctxt).unwrap(),
            Some(6)
        );
    }
}
