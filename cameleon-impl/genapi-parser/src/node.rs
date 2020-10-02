use super::{node_base::*, xml, Parse};

#[derive(Debug, Clone)]
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
}

impl Parse for Node {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "Node");

        let attr_base = NodeAttributeBase::parse(node);
        let elem_base = NodeElementBase::parse(node);

        let p_invalidators = node.parse_while("pInvalidator");

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

    fn node_from_str(xml: &str) -> Node {
        let document = xml::Document::from_str(xml).unwrap();
        document.root_node().parse()
    }

    #[test]
    fn test_all_fields_filled() {
        let xml = r#"
            <Node Name = "TestNode" NameSpace = "Standard" MergePriority = "1" ExposeStatic = "No">
                <ToolTip>tooltip</ToolTip>
                <Description>the description</Description>
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
                <pError>AnotherErr0</pError>
                <pError>AnotherErr1</pError>
                <pAlias>AnotherNode5</pAlias>
                <pCastAlias>AnotherNode6</pCastAlias>
                <pInvalidator>Invalidator0</pInvalidator>
                <pInvalidator>Invalidator1</pInvalidator>
            </Node>
            "#;

        let node = node_from_str(&xml);
        let node_base = node.node_base();
        assert_eq!(node_base.name(), "TestNode");
        assert_eq!(node_base.name_space(), NameSpace::Standard);
        assert_eq!(node_base.merge_priority(), MergePriority::High);
        assert_eq!(node_base.expose_static().unwrap(), false);

        assert_eq!(node_base.tool_tip().unwrap(), "tooltip");
        assert_eq!(node_base.description().unwrap(), "the description");
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
        assert_eq!(node_base.p_errors().len(), 2);
        assert_eq!(node_base.p_errors()[0], "AnotherErr0");
        assert_eq!(node_base.p_errors()[1], "AnotherErr1");
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

        let node = node_from_str(&xml);
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
        assert_eq!(node_base.p_errors().len(), 0);
        assert!(node_base.p_alias().is_none());
        assert!(node_base.p_cast_alias().is_none());

        let p_invalidators = node.p_invalidators();
        assert!(p_invalidators.is_empty());
    }
}
