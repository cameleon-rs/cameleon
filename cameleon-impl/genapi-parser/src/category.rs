use super::{node_base::*, xml};

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

    pub(super) fn parse(mut node: xml::Node) -> Self {
        debug_assert!(node.tag_name() == "Category");

        let attr_base = NodeAttributeBase::parse(&node);
        let elem_base = NodeElementBase::parse(&mut node);

        let mut p_invalidators = vec![];
        while let Some(text) = node.next_text_if("pInvalidator") {
            p_invalidators.push(text)
        }

        let mut p_features = vec![];
        while let Some(text) = node.next_text_if("pFeature") {
            p_features.push(text)
        }

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

        let xml_parser = libxml::parser::Parser::default();
        let document = xml_parser.parse_string(&xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.get_root_element().unwrap());
        let node = CategoryNode::parse(node);

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

        let xml_parser = libxml::parser::Parser::default();
        let document = xml_parser.parse_string(&xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.get_root_element().unwrap());
        let node = CategoryNode::parse(node);

        let p_invalidators = node.p_invalidators();
        assert!(p_invalidators.is_empty());

        let p_features = node.p_features();
        assert!(p_features.is_empty());
    }
}
