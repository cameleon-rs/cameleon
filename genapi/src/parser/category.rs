use crate::{
    store::{NodeStore, ValueStore},
    CategoryNode,
};

use super::{
    elem_name::{CATEGORY, P_FEATURE},
    xml, Parse,
};

impl Parse for CategoryNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), CATEGORY);

        let attr_base = node.parse(node_store, value_store);
        let elem_base = node.parse(node_store, value_store);

        let p_features = node.parse_while(P_FEATURE, node_store, value_store);

        Self {
            attr_base,
            elem_base,
            p_features,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::store::{DefaultNodeStore, DefaultValueStore};

    use super::*;

    fn category_node_from_str(xml: &str) -> (CategoryNode, DefaultNodeStore, DefaultValueStore) {
        let document = xml::Document::from_str(xml).unwrap();
        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        (
            document
                .root_node()
                .parse(&mut node_store, &mut value_store),
            node_store,
            value_store,
        )
    }

    #[test]
    fn test_category_node_filled() {
        let xml = r#"
            <Category Name = "TestNode">
                <pFeature>FeatureNode0</pFeature>
                <pFeature>FeatureNode1</pFeature>
            </Category>
            "#;

        let (node, mut node_store, _) = category_node_from_str(&xml);

        let p_features = node.p_features();
        assert_eq!(p_features.len(), 2);
        assert_eq!(p_features[0], node_store.id_by_name("FeatureNode0"));
        assert_eq!(p_features[1], node_store.id_by_name("FeatureNode1"));
    }

    #[test]
    fn test_category_default() {
        let xml = r#"
            <Category Name = "TestNode">
            </Category>
            "#;

        let (node, ..) = category_node_from_str(&xml);

        let p_features = node.p_features();
        assert!(p_features.is_empty());
    }
}
