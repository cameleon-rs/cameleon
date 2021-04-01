use super::{
    elem_name::{
        CONSTANT, EXPRESSION, INT_CONVERTER, P_INVALIDATOR, P_VARIABLE, REPRESENTATION, SLOPE,
        STREAMABLE, UNIT,
    },
    elem_type::{IntegerRepresentation, NamedValue, Slope},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct IntConverterNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    streamable: bool,
    p_variables: Vec<NamedValue<NodeId>>,
    constants: Vec<NamedValue<i64>>,
    expressions: Vec<NamedValue<String>>,
    formula_to: String,
    formula_from: String,
    p_value: NodeId,
    unit: Option<String>,
    representation: IntegerRepresentation,
    slope: Slope,
}

impl IntConverterNode {
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
    pub fn constants(&self) -> &[NamedValue<i64>] {
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
    pub fn representation(&self) -> IntegerRepresentation {
        self.representation
    }

    #[must_use]
    pub fn slope(&self) -> Slope {
        self.slope
    }
}

impl Parse for IntConverterNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), INT_CONVERTER);

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
        let slope = node.parse_if(SLOPE, store).unwrap_or_default();

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
            slope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_converter() {
        let xml = r#"
            <IntConverter Name="Testnode">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <FormulaTo>FROM*Var1/Var2</FormulaTo>
                <FormulaFrom>TO/Var1*Var2</FormulaFrom>
                <pValue>Target</pValue>
             </IntConverter>
             "#;

        let mut store = NodeStore::new();
        let node: IntConverterNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        let p_variables = node.p_variables();
        assert_eq!(p_variables.len(), 2);
        assert_eq!(p_variables[0].name(), "Var1");
        assert_eq!(p_variables[0].value(), &store.id_by_name("pValue1"));
        assert_eq!(p_variables[1].name(), "Var2");
        assert_eq!(p_variables[1].value(), &store.id_by_name("pValue2"));

        assert_eq!(node.formula_to(), "FROM*Var1/Var2");
        assert_eq!(node.formula_from(), "TO/Var1*Var2");
        assert_eq!(node.p_value(), store.id_by_name("Target"));
        assert_eq!(node.representation(), IntegerRepresentation::PureNumber);
        assert_eq!(node.slope(), Slope::Automatic);
    }
}
