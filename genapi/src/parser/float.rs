use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    elem_type::ImmOrPNode,
    FloatNode,
};

use super::{
    elem_name::{
        DISPLAY_NOTATION, DISPLAY_PRECISION, FLOAT, INC, MAX, MIN, P_INC, P_MAX, P_MIN,
        REPRESENTATION, STREAMABLE, UNIT,
    },
    xml, Parse,
};

impl Parse for FloatNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `FloatNode`");
        debug_assert_eq!(node.tag_name(), FLOAT);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let streamable = node
            .parse_if(STREAMABLE, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let value_kind = node.parse(node_builder, value_builder, cache_builder);
        let min = node
            .parse_if(MIN, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(P_MIN, node_builder, value_builder, cache_builder))
            .unwrap_or_else(|| {
                let id = value_builder.store(f64::MIN);
                ImmOrPNode::Imm(id)
            });
        let max = node
            .parse_if(MAX, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(P_MAX, node_builder, value_builder, cache_builder))
            .unwrap_or_else(|| {
                let id = value_builder.store(f64::MAX);
                ImmOrPNode::Imm(id)
            });
        let inc = node
            .parse_if(INC, node_builder, value_builder, cache_builder)
            .or_else(|| node.parse_if(P_INC, node_builder, value_builder, cache_builder));
        let unit = node.parse_if(UNIT, node_builder, value_builder, cache_builder);
        let representation = node
            .parse_if(REPRESENTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let display_notation = node
            .parse_if(DISPLAY_NOTATION, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let display_precision = node
            .parse_if(
                DISPLAY_PRECISION,
                node_builder,
                value_builder,
                cache_builder,
            )
            .unwrap_or(6);

        Self {
            attr_base,
            elem_base,
            streamable,
            value_kind,
            min,
            max,
            inc,
            unit,
            representation,
            display_notation,
            display_precision,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        elem_type::{DisplayNotation, FloatRepresentation, ValueKind},
        store::ValueStore,
    };

    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_float_node() {
        let xml = r#"
            <Float Name = "TestNode">
                <Streamable>Yes</Streamable>
                <Value>-.45E-6</Value>
                <Min>-INF</Min>
                <Max>INF</Max>
                <Inc>NaN</Inc>
                <Unit>dB</Unit>
                <Representation>Logarithmic</Representation>
                <DisplayNotation>Fixed</DisplayNotation>
                <DisplayPrecision>10</DisplayPrecision>
            </Float>
            "#;

        let (node, _, value_builder, _): (FloatNode, _, _, _) = parse_default(xml);

        assert!(node.streamable());
        assert!(matches! {node.value_kind(), ValueKind::Value(_)});
        let min_value = value_builder
            .float_value(node.min_elem().imm().unwrap())
            .unwrap();
        assert!(min_value.is_infinite() && min_value.is_sign_negative());
        let max_value = value_builder
            .float_value(node.max_elem().imm().unwrap())
            .unwrap();
        assert!(max_value.is_infinite() && max_value.is_sign_positive());
        assert!(node.inc_elem().unwrap().imm().unwrap().is_nan());
        assert_eq!(node.unit_elem(), Some("dB"));
        assert_eq!(node.representation_elem(), FloatRepresentation::Logarithmic);
        assert_eq!(node.display_notation_elem(), DisplayNotation::Fixed);
        assert_eq!(node.display_precision_elem(), 10);
    }
}
