use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    CategoryNode,
};

use super::{
    elem_name::{CATEGORY, P_FEATURE},
    xml, Parse,
};

impl Parse for CategoryNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `CategoryNode`");
        debug_assert_eq!(node.tag_name(), CATEGORY);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let p_features = node.parse_while(P_FEATURE, node_builder, value_builder, cache_builder);

        Self {
            attr_base,
            elem_base,
            p_features,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_category_node_filled() {
        let xml = r#"
            <Category Name = "TestNode">
                <pFeature>FeatureNode0</pFeature>
                <pFeature>FeatureNode1</pFeature>
            </Category>
            "#;

        let (node, mut node_builder, ..): (CategoryNode, _, _, _) = parse_default(xml);
        let p_features = node.p_features();
        assert_eq!(p_features.len(), 2);
        assert_eq!(p_features[0], node_builder.get_or_intern("FeatureNode0"));
        assert_eq!(p_features[1], node_builder.get_or_intern("FeatureNode1"));
    }

    #[test]
    fn test_category_default() {
        let xml = r#"
            <Category Name = "TestNode">
            </Category>
            "#;

        let (node, ..): (CategoryNode, _, _, _) = parse_default(xml);
        let p_features = node.p_features();
        assert!(p_features.is_empty());
    }
}
