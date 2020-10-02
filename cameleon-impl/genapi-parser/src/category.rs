use super::{node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct CategoryNode {
    attr_base: NodeAttributeBase,

    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,

    p_features: Vec<String>,
}

impl CategoryNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn p_features(&self) -> &[String] {
        &self.p_features
    }
}

impl Parse for CategoryNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "Category");

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while("pInvalidator");

        let p_features = node.parse_while("pFeature");

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

    fn category_node_from_str(xml: &str) -> CategoryNode {
        let document = xml::Document::from_str(xml).unwrap();
        document.root_node().parse()
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

        let node = category_node_from_str(&xml);

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0].as_str(), "Invalidator0");
        assert_eq!(p_invalidators[1].as_str(), "Invalidator1");

        let p_features = node.p_features();
        assert_eq!(p_features.len(), 2);
        assert_eq!(p_features[0].as_str(), "FeatureNode0");
        assert_eq!(p_features[1].as_str(), "FeatureNode1");
    }

    #[test]
    fn test_category_default() {
        let xml = r#"
            <Category Name = "TestNode">
            </Category>
            "#;

        let node = category_node_from_str(&xml);

        let p_invalidators = node.p_invalidators();
        assert!(p_invalidators.is_empty());

        let p_features = node.p_features();
        assert!(p_features.is_empty());
    }
}
