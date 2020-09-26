use super::{node_base::*, xml};

pub struct Node {
    attr_base: NodeAttributeBase,

    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,
}

impl Node {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub(super) fn parse(mut node: xml::Node) -> Self {
        debug_assert!(node.tag_name() == "Node");

        let attr_base = NodeAttributeBase::parse(&node);
        let elem_base = NodeElementBase::parse(&mut node);

        let mut p_invalidators = vec![];
        while let Some(text) = node.next_text_if("pInvalidator") {
            p_invalidators.push(text)
        }

        Self {
            attr_base,
            elem_base,
            p_invalidators,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::elem_type::*;

    use super::*;

    #[test]
    fn test_all_fields_filled() {
        let xml = r#"
            <Node Name = "TestNode" NameSpace = "Standard" MergePriority = "1" ExposeStatic = "No">
                <ToolTip>tooltip</ToolTip>
                <DisplayName>display name</DisplayName>
                <Visibility>Guru</Visibility>
                <DocuURL>http://FOO.com</DocuURL>
                <IsDeprecated>Yes</IsDeprecated>
                <EventID>F1</EventID>
                <pIsImplemented>AnotherNode0</pIsImplemented>
                <pIsAvailable>AnotherNode1</pIsAvailable>
                <pIsLocked>AnotherNode2</pIsLocked>
                <pBlockPolling>AnotherNode3</pBlockPolling>
                <ImposedAccessMode>RO</ImposedAccessMode>
                <pError>AnotherNode4</pError>
                <pAlias>AnotherNode5</pAlias>
                <pCastAlias>AnotherNode6</pCastAlias>
                <pInvalidator>Invalidator0</pInvalidator>
                <pInvalidator>Invalidator1</pInvalidator>
            </Node>
            "#;

        let xml_parser = libxml::parser::Parser::default();
        let document = xml_parser.parse_string(&xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.get_root_element().unwrap());
        let node = Node::parse(node);

        let node_base = node.node_base();
        assert_eq!(node_base.name(), "TestNode");
        assert_eq!(node_base.name_space(), NameSpace::Standard);
        assert_eq!(node_base.merge_priority(), MergePriority::High);
        assert_eq!(node_base.expose_static().unwrap(), false);

        assert_eq!(node_base.tool_tip().unwrap(), "tooltip");
        assert_eq!(node_base.display_name(), "display name");
        assert_eq!(node_base.visibility(), Visibility::Guru);
        assert_eq!(node_base.docu_url().unwrap(), "http://FOO.com");
        assert_eq!(node_base.is_deprecated(), true);
        assert_eq!(node_base.event_id().unwrap(), "F1");
        assert_eq!(node_base.p_is_implemented().unwrap(), "AnotherNode0");
        assert_eq!(node_base.p_is_available().unwrap(), "AnotherNode1");
        assert_eq!(node_base.p_is_locked().unwrap(), "AnotherNode2");
        assert_eq!(node_base.p_block_polling().unwrap(), "AnotherNode3");
        assert_eq!(node_base.imposed_access_mode(), AccessMode::RO);
        assert_eq!(node_base.p_error().unwrap(), "AnotherNode4");
        assert_eq!(node_base.p_alias().unwrap(), "AnotherNode5");
        assert_eq!(node_base.p_cast_alias().unwrap(), "AnotherNode6");

        let p_invalidators = node.p_invalidators();
        assert_eq!(p_invalidators.len(), 2);
        assert_eq!(p_invalidators[0].as_str(), "Invalidator0");
        assert_eq!(p_invalidators[1].as_str(), "Invalidator1");
    }

    #[test]
    fn test_default() {
        let xml = r#"
            <Node Name = "TestNode">
            </Node>
            "#;

        let parser = libxml::parser::Parser::default();
        let document = parser.parse_string(&xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.get_root_element().unwrap());
        let node = Node::parse(node);

        let node_base = node.node_base();
        assert_eq!(node_base.name(), "TestNode");
        assert_eq!(node_base.name_space(), NameSpace::Custom);
        assert_eq!(node_base.merge_priority(), MergePriority::Mid);
        assert!(node_base.expose_static().is_none());

        assert!(node_base.tool_tip().is_none());
        assert_eq!(node_base.display_name(), "TestNode");
        assert_eq!(node_base.visibility(), Visibility::Beginner);
        assert!(node_base.docu_url().is_none());
        assert_eq!(node_base.is_deprecated(), false);
        assert!(node_base.event_id().is_none());
        assert!(node_base.p_is_implemented().is_none());
        assert!(node_base.p_is_available().is_none());
        assert!(node_base.p_is_locked().is_none());
        assert!(node_base.p_block_polling().is_none());
        assert_eq!(node_base.imposed_access_mode(), AccessMode::RW);
        assert!(node_base.p_error().is_none());
        assert!(node_base.p_alias().is_none());
        assert!(node_base.p_cast_alias().is_none());

        let p_invalidators = node.p_invalidators();
        assert!(p_invalidators.is_empty());
    }
}
