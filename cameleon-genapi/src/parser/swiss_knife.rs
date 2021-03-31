use super::{
    elem_name::{
        CONSTANT, DISPLAY_NOTATION, DISPLAY_PRECISION, EXPRESSION, P_INVALIDATOR, P_VARIABLE,
        REPRESENTATION, STREAMABLE, SWISS_KNIFE, UNIT,
    },
    elem_type::{DisplayNotation, FloatRepresentation, NamedValue},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct SwissKnifeNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,
    streamable: bool,
    p_variables: Vec<NamedValue<String>>,
    constants: Vec<NamedValue<f64>>,
    expressions: Vec<NamedValue<String>>,
    formula: String,
    unit: Option<String>,
    representation: FloatRepresentation,
    display_notation: DisplayNotation,
    display_precision: i64,
}

impl SwissKnifeNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn p_variables(&self) -> &[NamedValue<String>] {
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
    pub fn formula(&self) -> &str {
        &self.formula
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
}

impl Parse for SwissKnifeNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), SWISS_KNIFE);

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while(P_INVALIDATOR);
        let streamable = node.parse_if(STREAMABLE).unwrap_or_default();
        let p_variables = node.parse_while(P_VARIABLE);
        let constants = node.parse_while(CONSTANT);
        let expressions = node.parse_while(EXPRESSION);
        let formula = node.parse();
        let unit = node.parse_if(UNIT);
        let representation = node.parse_if(REPRESENTATION).unwrap_or_default();
        let display_notation = node.parse_if(DISPLAY_NOTATION).unwrap_or_default();
        let display_precision = node.parse_if(DISPLAY_PRECISION).unwrap_or(6);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            p_variables,
            constants,
            expressions,
            formula,
            unit,
            representation,
            display_notation,
            display_precision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swiss_knife() {
        let xml = r#"
            <SwissKnife Name="Testnode">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <Constant Name="Const">INF</Constant>
                <Expression Name="ConstBy2">2.0*Const</Expression>
                <Formula>Var1+Var2+ConstBy2</Formula>
             </SwissKnife>
             "#;

        let node: SwissKnifeNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        let p_variables = node.p_variables();
        assert_eq!(p_variables.len(), 2);
        assert_eq!(p_variables[0].name(), "Var1");
        assert_eq!(p_variables[0].value(), "pValue1");
        assert_eq!(p_variables[1].name(), "Var2");
        assert_eq!(p_variables[1].value(), "pValue2");

        let constants = node.constants();
        assert_eq!(constants.len(), 1);
        assert_eq!(constants[0].name(), "Const");
        assert!(constants[0].value().is_infinite());

        let expressions = node.expressions();
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].name(), "ConstBy2");
        assert_eq!(expressions[0].value(), "2.0*Const");

        assert_eq!(node.formula(), "Var1+Var2+ConstBy2");
    }
}
