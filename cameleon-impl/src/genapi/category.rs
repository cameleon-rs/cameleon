use super::{node_base::*, verifier::verify_node_name, xml, GenApiResult, Span};

pub struct CategoryNode {
    attr_base: NodeAttributeBase,

    elem_base: NodeElementBase,

    p_invalidators: Vec<Span<String>>,

    p_features: Vec<Span<String>>,
}

impl CategoryNode {
    pub fn node_base<'a>(&'a self) -> NodeBase<'a> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[Span<String>] {
        &self.p_invalidators
    }

    pub fn p_features(&self) -> &[Span<String>] {
        &self.p_features
    }
}

impl CategoryNode {
    pub(super) fn parse(mut node: Span<xml::Node>) -> GenApiResult<Self> {
        debug_assert!(node.tag_name() == "Category");

        let attr_base = NodeAttributeBase::parse(&mut node)?;
        let elem_base = NodeElementBase::parse(&mut node)?;

        let mut p_invalidators = vec![];
        while let Some(text) = node.next_child_elem_text_if("pInvalidator")? {
            verify_node_name(text)?;
            p_invalidators.push(text.map(Into::into))
        }

        let mut p_features = vec![];
        while let Some(text) = node.next_child_elem_text_if("pFeature")? {
            verify_node_name(text)?;
            p_features.push(text.map(Into::into))
        }

        Ok(Self {
            attr_base,
            elem_base,
            p_invalidators,
            p_features,
        })
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

        let document = roxmltree::Document::parse(xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.root_element());
        let node = CategoryNode::parse(node).unwrap();

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

        let document = roxmltree::Document::parse(xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.root_element());
        let node = CategoryNode::parse(node).unwrap();

        let p_invalidators = node.p_invalidators();
        assert!(p_invalidators.is_empty());

        let p_features = node.p_features();
        assert!(p_features.is_empty());
    }
}
