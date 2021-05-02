use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    IntSwissKnifeNode,
};

use super::{
    elem_name::{
        CONSTANT, EXPRESSION, INT_SWISS_KNIFE, P_VARIABLE, REPRESENTATION, STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for IntSwissKnifeNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `IntSwissKnifeNode`");
        debug_assert_eq!(node.tag_name(), INT_SWISS_KNIFE);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let p_variables = node.parse_while(P_VARIABLE, node_builder, value_builder, cache_builder);
        let constants = node.parse_while(CONSTANT, node_builder, value_builder, cache_builder);
        let expressions = node.parse_while(EXPRESSION, node_builder, value_builder, cache_builder);
        let formula = node.parse(node_builder, value_builder, cache_builder);
        let unit = node.parse_if(UNIT, node_builder, value_builder, cache_builder);
        let representation = node
            .parse_if(REPRESENTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();

        Self {
            attr_base,
            elem_base,
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
    use super::{super::utils::tests::parse_default, *};

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

        let (node, mut node_builder, ..): (IntSwissKnifeNode, _, _, _) = parse_default(xml);

        let p_variables = node.p_variables();
        assert_eq!(p_variables.len(), 2);
        assert_eq!(p_variables[0].name(), "Var1");
        assert_eq!(p_variables[0].value(), node_builder.get_or_intern("pValue1"));
        assert_eq!(p_variables[1].name(), "Var2");
        assert_eq!(p_variables[1].value(), node_builder.get_or_intern("pValue2"));

        let constants = node.constants();
        assert_eq!(constants.len(), 1);
        assert_eq!(constants[0].name(), "Const");
        assert_eq!(constants[0].value(), 10);

        let expressions = node.expressions();
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].name(), "ConstBy2");
    }
}
