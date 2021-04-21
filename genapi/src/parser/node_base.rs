use crate::{
    elem_type::AccessMode,
    node_base::{NodeAttributeBase, NodeElementBase},
    store::{NodeStore, ValueStore},
};

use super::{
    elem_name::{
        DESCRIPTION, DISPLAY_NAME, DOCU_URL, EVENT_ID, EXPOSE_STATIC, EXTENSION,
        IMPOSED_ACCESS_MODE, IS_DEPRECATED, MERGE_PRIORITY, NAME, NAME_SPACE, P_ALIAS,
        P_BLOCK_POLLING, P_CAST_ALIAS, P_ERROR, P_IS_AVAILABLE, P_IS_IMPLEMENTED, P_IS_LOCKED,
        TOOL_TIP, VISIBILITY,
    },
    elem_type::convert_to_bool,
    xml, Parse,
};

impl Parse for NodeAttributeBase {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, _: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        let name = node.attribute_of(NAME).unwrap();
        let id = node_store.id_by_name(&name);
        let name_space = node
            .attribute_of(NAME_SPACE)
            .map(|text| text.into())
            .unwrap_or_default();
        let merge_priority = node
            .attribute_of(MERGE_PRIORITY)
            .map(|text| text.into())
            .unwrap_or_default();
        let expose_static = node
            .attribute_of(EXPOSE_STATIC)
            .map(|text| convert_to_bool(&text));

        Self {
            id,
            name_space,
            merge_priority,
            expose_static,
        }
    }
}

impl Parse for NodeElementBase {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        // Ignore Extension element.
        let _: Option<String> = node.parse_if(EXTENSION, node_store, value_store);

        let tool_tip = node.parse_if(TOOL_TIP, node_store, value_store);
        let description = node.parse_if(DESCRIPTION, node_store, value_store);
        let display_name = node.parse_if(DISPLAY_NAME, node_store, value_store);
        let visibility = node
            .parse_if(VISIBILITY, node_store, value_store)
            .unwrap_or_default();
        let docu_url = node.parse_if(DOCU_URL, node_store, value_store);
        let is_deprecated = node
            .parse_if(IS_DEPRECATED, node_store, value_store)
            .unwrap_or_default();
        let event_id = node
            .next_if(EVENT_ID)
            .map(|n| u64::from_str_radix(n.text(), 16).unwrap());
        let p_is_implemented = node.parse_if(P_IS_IMPLEMENTED, node_store, value_store);
        let p_is_available = node.parse_if(P_IS_AVAILABLE, node_store, value_store);
        let p_is_locked = node.parse_if(P_IS_LOCKED, node_store, value_store);
        let p_block_polling = node.parse_if(P_BLOCK_POLLING, node_store, value_store);
        let imposed_access_mode = node
            .parse_if(IMPOSED_ACCESS_MODE, node_store, value_store)
            .unwrap_or(AccessMode::RW);
        let p_errors = node.parse_while(P_ERROR, node_store, value_store);
        let p_alias = node.parse_if(P_ALIAS, node_store, value_store);
        let p_cast_alias = node.parse_if(P_CAST_ALIAS, node_store, value_store);

        Self {
            tool_tip,
            description,
            display_name,
            visibility,
            docu_url,
            is_deprecated,
            event_id,
            p_is_implemented,
            p_is_available,
            p_is_locked,
            p_block_polling,
            imposed_access_mode,
            p_errors,
            p_alias,
            p_cast_alias,
        }
    }
}
