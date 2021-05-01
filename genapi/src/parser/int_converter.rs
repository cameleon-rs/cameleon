use tracing::debug;

use crate::{
    store::{ValueStore, WritableNodeStore},
    IntConverterNode,
};

use super::{
    elem_name::{
        CONSTANT, EXPRESSION, INT_CONVERTER, P_VARIABLE, REPRESENTATION, SLOPE, STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for IntConverterNode {
    #[tracing::instrument(level = "trace", skip(node_store, value_store))]
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug!("start parsing `IntConverterNode`");
        debug_assert_eq!(node.tag_name(), INT_CONVERTER);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let streamable = node
            .parse_if(STREAMABLE, node_store, value_store)
            .unwrap_or_default();
        let p_variables = node.parse_while(P_VARIABLE, node_store, value_store);
        let constants = node.parse_while(CONSTANT, node_store, value_store);
        let expressions = node.parse_while(EXPRESSION, node_store, value_store);
        let formula_to = node.parse(node_store, value_store);
        let formula_from = node.parse(node_store, value_store);
        let p_value = node.parse(node_store, value_store);
        let unit = node.parse_if(UNIT, node_store, value_store);
        let representation = node
            .parse_if(REPRESENTATION, node_store, value_store)
            .unwrap_or_default();
        let slope = node
            .parse_if(SLOPE, node_store, value_store)
            .unwrap_or_default();

        Self {
            attr_base,
            elem_base,
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
    use crate::{
        elem_type::{IntegerRepresentation, Slope},
        store::{DefaultNodeStore, DefaultValueStore},
    };

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

        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let node: IntConverterNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut node_store, &mut value_store);

        let p_variables = node.p_variables();
        assert_eq!(p_variables.len(), 2);
        assert_eq!(p_variables[0].name(), "Var1");
        assert_eq!(p_variables[0].value(), node_store.id_by_name("pValue1"));
        assert_eq!(p_variables[1].name(), "Var2");
        assert_eq!(p_variables[1].value(), node_store.id_by_name("pValue2"));

        assert_eq!(node.p_value(), node_store.id_by_name("Target"));
        assert_eq!(
            node.representation_elem(),
            IntegerRepresentation::PureNumber
        );
        assert_eq!(node.slope(), Slope::Automatic);
    }
}
