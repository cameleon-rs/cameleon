use super::{elem_type::*, node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct IntSwissKnifeNode {
    attr_base: NodeAttributeBase,

    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,

    streamable: bool,

    p_variables: Vec<NamedValue<String>>,

    constants: Vec<NamedValue<i64>>,

    expressions: Vec<NamedValue<String>>,

    formula: String,

    unit: Option<String>,

    representation: IntegerRepresentation,
}

impl IntSwissKnifeNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn p_variables(&self) -> &[NamedValue<String>] {
        &self.p_variables
    }

    pub fn constants(&self) -> &[NamedValue<i64>] {
        &self.constants
    }

    pub fn expressions(&self) -> &[NamedValue<String>] {
        &self.expressions
    }

    pub fn formula(&self) -> &str {
        &self.formula
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }
}

impl Parse for IntSwissKnifeNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "IntSwissKnife");

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while("pInvalidator");

        let streamable = node.parse_if("Streamable").unwrap_or_default();

        let p_variables = node.parse_while("pVariable");

        let constants = node.parse_while("Constant");

        let expressions = node.parse_while("Expression");

        let formula = node.parse();

        let unit = node.parse_if("Unit");

        let representation = node.parse_if("Representation").unwrap_or_default();

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_swiss_knife() {
        let xml = r#"
            <IntSwissKnife Name="Testnode">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <Constant Name="Const">10</Constant>
                <Expression Name="ConstBy2">2.0*Const</Expression>
                <Formula>Var1+Var2+ConstBy2</Formula>
             </IntSwissKnife>
             "#;

        let node: IntSwissKnifeNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        let p_variables = node.p_variables();
        assert_eq!(p_variables.len(), 2);
        assert_eq!(p_variables[0].name(), "Var1");
        assert_eq!(p_variables[0].value(), "pValue1");
        assert_eq!(p_variables[1].name(), "Var2");
        assert_eq!(p_variables[1].value(), "pValue2");

        let constants = node.constants();
        assert_eq!(constants.len(), 1);
        assert_eq!(constants[0].name(), "Const");
        assert_eq!(*constants[0].value(), 10);

        let expressions = node.expressions();
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].name(), "ConstBy2");
        assert_eq!(expressions[0].value(), "2.0*Const");

        assert_eq!(node.formula(), "Var1+Var2+ConstBy2");
    }
}
