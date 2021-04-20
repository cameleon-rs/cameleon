use crate::{store::NodeStore, CategoryNode};

use super::{
    elem_name::{CATEGORY, P_FEATURE, P_INVALIDATOR},
    xml, Parse,
};

impl Parse for CategoryNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), CATEGORY);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let p_features = node.parse_while(P_FEATURE, store);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            p_features,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category_node_from_str(xml: &str) -> (CategoryNode, NodeStore) {
        let document = xml::Document::from_str(xml).unwrap();
        let mut store = NodeStore::new();
        (document.root_node().parse(&mut store), store)
    }

    #[test]
    fn test_category_node_filled() {
        let xml = r#"
            <Category Name = "TestNode">
                <pInvalidator>Invalidator0</pInvalidator>
                <pInvalidator>Invalidator1</pInvalidator>
                <pFeature>FeatureNode0</pFeature>
                <pFeature>FeatureNode1</pFeature>
            </Category>
            "#;

        let (node, mut store) = category_node_from_str(&xml);

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0], store.id_by_name("Invalidator0"));
        assert_eq!(p_invalidators[1], store.id_by_name("Invalidator1"));

        let p_features = node.p_features();
        assert_eq!(p_features.len(), 2);
        assert_eq!(p_features[0], store.id_by_name("FeatureNode0"));
        assert_eq!(p_features[1], store.id_by_name("FeatureNode1"));
    }

    #[test]
    fn test_category_default() {
        let xml = r#"
            <Category Name = "TestNode">
            </Category>
            "#;

        let (node, _) = category_node_from_str(&xml);

        let p_invalidators = node.p_invalidators();
        assert!(p_invalidators.is_empty());

        let p_features = node.p_features();
        assert!(p_features.is_empty());
    }
}
