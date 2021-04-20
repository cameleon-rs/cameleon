use crate::{store::NodeStore, IntConverterNode};

use super::{
    elem_name::{
        CONSTANT, EXPRESSION, INT_CONVERTER, P_INVALIDATOR, P_VARIABLE, REPRESENTATION, SLOPE,
        STREAMABLE, UNIT,
    },
    xml, Parse,
};

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
    use crate::elem_type::{IntegerRepresentation, Slope};

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
