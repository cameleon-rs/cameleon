use crate::{
    node_base::{NodeAttributeBase, NodeElementBase},
    store::{WritableNodeStore, ValueStore},
    Node,
};

use super::{elem_name::NODE, xml, Parse};

impl Parse for Node {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert_eq!(node.tag_name(), NODE);

        let attr_base = NodeAttributeBase::parse(node, node_store, value_store);
        let elem_base = NodeElementBase::parse(node, node_store, value_store);

        Self {
            attr_base,
            elem_base,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        elem_type::{AccessMode, MergePriority, NameSpace, Visibility},
        store::{DefaultNodeStore, DefaultValueStore},
    };

    use super::*;

    fn node_from_str(xml: &str) -> (Node, DefaultNodeStore, DefaultValueStore) {
        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let document = xml::Document::from_str(xml).unwrap();
        (
            document
                .root_node()
                .parse(&mut node_store, &mut value_store),
            node_store,
            value_store,
        )
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
            </Node>
            "#;

        let (node, mut node_store, _) = node_from_str(&xml);
        let node_base = node.node_base();
        assert_eq!(node_base.id(), node_store.id_by_name("TestNode"));
        assert_eq!(node_base.name_space(), NameSpace::Standard);
        assert_eq!(node_base.merge_priority(), MergePriority::High);
        assert_eq!(node_base.expose_static().unwrap(), false);

        assert_eq!(node_base.tool_tip().unwrap(), "tooltip");
        assert_eq!(node_base.description().unwrap(), "the description");
        assert_eq!(node_base.display_name(), Some("display name"));
        assert_eq!(node_base.visibility(), Visibility::Guru);
        assert_eq!(node_base.docu_url().unwrap(), "http://FOO.com");
        assert_eq!(node_base.is_deprecated(), true);
        assert_eq!(node_base.event_id(), Some(0xF1));
        assert_eq!(
            node_base.p_is_implemented().unwrap(),
            node_store.id_by_name("AnotherNode0")
        );
        assert_eq!(
            node_base.p_is_available().unwrap(),
            node_store.id_by_name("AnotherNode1")
        );
        assert_eq!(
            node_base.p_is_locked().unwrap(),
            node_store.id_by_name("AnotherNode2")
        );
        assert_eq!(
            node_base.p_block_polling().unwrap(),
            node_store.id_by_name("AnotherNode3")
        );
        assert_eq!(node_base.imposed_access_mode(), AccessMode::RO);
        assert_eq!(node_base.p_errors().len(), 2);
        assert_eq!(
            node_base.p_errors()[0],
            node_store.id_by_name("AnotherErr0")
        );
        assert_eq!(
            node_base.p_errors()[1],
            node_store.id_by_name("AnotherErr1")
        );
        assert_eq!(
            node_base.p_alias().unwrap(),
            node_store.id_by_name("AnotherNode5")
        );
        assert_eq!(
            node_base.p_cast_alias().unwrap(),
            node_store.id_by_name("AnotherNode6")
        );
    }

    #[test]
    fn test_default() {
        let xml = r#"
            <Node Name = "TestNode">
            </Node>
            "#;

        let (node, mut node_store, _) = node_from_str(&xml);
        let node_base = node.node_base();
        assert_eq!(node_base.id(), node_store.id_by_name("TestNode"));
        assert_eq!(node_base.name_space(), NameSpace::Custom);
        assert_eq!(node_base.merge_priority(), MergePriority::Mid);
        assert!(node_base.expose_static().is_none());

        assert!(node_base.tool_tip().is_none());
        assert_eq!(node_base.display_name(), None);
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
    }
}
