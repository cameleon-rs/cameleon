use super::{
    elem_name::{
        CONSTANT, CONVERTER, DISPLAY_NOTATION, DISPLAY_PRECISION, EXPRESSION, IS_LINEAR,
        P_INVALIDATOR, P_VARIABLE, REPRESENTATION, SLOPE, STREAMABLE, UNIT,
    },
    elem_type::{DisplayNotation, FloatRepresentation, NamedValue, Slope},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct ConverterNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    streamable: bool,
    p_variables: Vec<NamedValue<NodeId>>,
    constants: Vec<NamedValue<f64>>,
    expressions: Vec<NamedValue<String>>,
    formula_to: String,
    formula_from: String,
    p_value: NodeId,
    unit: Option<String>,
    representation: FloatRepresentation,
    display_notation: DisplayNotation,
    display_precision: i64,
    slope: Slope,
    is_linear: bool,
}

impl ConverterNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
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
    pub fn constants(&self) -> &[NamedValue<f64>] {
        &self.constants
    }

    #[must_use]
    pub fn expressions(&self) -> &[NamedValue<String>] {
        &self.expressions
    }

    #[must_use]
    pub fn formula_to(&self) -> &str {
        &self.formula_to
    }

    #[must_use]
    pub fn formula_from(&self) -> &str {
        &self.formula_from
    }

    #[must_use]
    pub fn p_value(&self) -> NodeId {
        self.p_value
    }

    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    #[must_use]
    pub fn representation(&self) -> FloatRepresentation {
        self.representation
    }

    #[must_use]
    pub fn display_notation(&self) -> DisplayNotation {
        self.display_notation
    }

    #[must_use]
    pub fn display_precision(&self) -> i64 {
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

impl Parse for ConverterNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), CONVERTER);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let p_variables = node.parse_while(P_VARIABLE, store);
        let constants = node.parse_while(CONSTANT, store);
        let expressions = node.parse_while(EXPRESSION, store);
        let formula_to = node.parse(store);
        let formula_from = node.parse(store);
        let p_value = node.parse(store);
        let unit = node.parse_if(UNIT, store);
        let representation = node.parse_if(REPRESENTATION, store).unwrap_or_default();
        let display_notation = node.parse_if(DISPLAY_NOTATION, store).unwrap_or_default();
        let display_precision = node.parse_if(DISPLAY_PRECISION, store).unwrap_or(6);
        let slope = node.parse_if(SLOPE, store).unwrap_or_default();
        let is_linear = node.parse_if(IS_LINEAR, store).unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            p_variables,
            constants,
            expressions,
            formula_to,
            formula_from,
            p_value,
            unit,
            representation,
            display_notation,
            display_precision,
            slope,
            is_linear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter() {
        let xml = r#"
            <Converter Name="Testnode">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <Constant Name="Const">INF</Constant>
                <Expression Name="ConstBy2">2.0*Const</Expression>
                <FormulaTo>FROM*Var1/Var2</FormulaTo>
                <FormulaFrom>TO/Var1*Var2</FormulaFrom>
                <pValue>Target</pValue>
                <Slope>Increasing</Slope>
                <IsLinear>Yes</IsLinear>
             </Converter>
             "#;

        let mut store = NodeStore::new();
        let node: ConverterNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        let p_variables = node.p_variables();
        assert_eq!(p_variables.len(), 2);
        assert_eq!(p_variables[0].name(), "Var1");
        assert_eq!(p_variables[0].value(), &store.id_by_name("pValue1"));
        assert_eq!(p_variables[1].name(), "Var2");
        assert_eq!(p_variables[1].value(), &store.id_by_name("pValue2"));

        let constants = node.constants();
        assert_eq!(constants.len(), 1);
        assert_eq!(constants[0].name(), "Const");
        assert!(constants[0].value().is_infinite());

        let expressions = node.expressions();
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].name(), "ConstBy2");
        assert_eq!(expressions[0].value(), "2.0*Const");

        assert_eq!(node.formula_to(), "FROM*Var1/Var2");
        assert_eq!(node.formula_from(), "TO/Var1*Var2");
        assert_eq!(node.p_value(), store.id_by_name("Target"));
        assert_eq!(node.slope(), Slope::Increasing);
        assert_eq!(node.is_linear(), true);
    }
}
