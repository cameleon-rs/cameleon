use super::{
    elem_name::{
        DESCRIPTION, DISPLAY_NAME, DOCU_URL, EVENT_ID, EXPOSE_STATIC, EXTENSION,
        IMPOSED_ACCESS_MODE, IS_DEPRECATED, MERGE_PRIORITY, NAME, NAME_SPACE, P_ALIAS,
        P_BLOCK_POLLING, P_CAST_ALIAS, P_ERROR, P_IS_AVAILABLE, P_IS_IMPLEMENTED, P_IS_LOCKED,
        TOOL_TIP, VISIBILITY,
    },
    elem_type::{convert_to_bool, AccessMode, MergePriority, NameSpace, Visibility},
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

pub struct NodeBase<'a> {
    attr: &'a NodeAttributeBase,
    elem: &'a NodeElementBase,
}

macro_rules! optional_string_elem_getter {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        #[must_use] pub fn $name(&self) -> Option<&'a str> {
            self.elem.$name.as_deref()
        }
    };
}

macro_rules! optional_node_id_elem_getter {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        #[must_use] pub fn $name(&self) -> Option<NodeId> {
            self.elem.$name
        }
    };
}

impl<'a> NodeBase<'a> {
    pub(super) fn new(attr: &'a NodeAttributeBase, elem: &'a NodeElementBase) -> Self {
        Self { attr, elem }
    }

    #[must_use]
    pub fn id(&self) -> NodeId {
        self.attr.id
    }

    #[must_use]
    pub fn name_space(&self) -> NameSpace {
        self.attr.name_space
    }

    #[must_use]
    pub fn merge_priority(&self) -> MergePriority {
        self.attr.merge_priority
    }

    #[must_use]
    pub fn expose_static(&self) -> Option<bool> {
        self.attr.expose_static
    }

    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.elem.display_name.as_deref()
    }

    #[must_use]
    pub fn visibility(&self) -> Visibility {
        self.elem.visibility
    }

    #[must_use]
    pub fn is_deprecated(&self) -> bool {
        self.elem.is_deprecated
    }

    #[must_use]
    pub fn imposed_access_mode(&self) -> AccessMode {
        self.elem.imposed_access_mode
    }

    #[must_use]
    pub fn p_errors(&self) -> &'a [NodeId] {
        &self.elem.p_errors
    }

    #[must_use]
    pub fn event_id(&self) -> Option<u64> {
        self.elem.event_id
    }

    optional_string_elem_getter! {description}
    optional_string_elem_getter! {tool_tip}
    optional_string_elem_getter! {docu_url}
    optional_node_id_elem_getter! {p_is_implemented}
    optional_node_id_elem_getter! {p_is_available}
    optional_node_id_elem_getter! {p_is_locked}
    optional_node_id_elem_getter! {p_block_polling}
    optional_node_id_elem_getter! {p_alias}
    optional_node_id_elem_getter! {p_cast_alias}
}

#[derive(Debug, Clone)]
pub(super) struct NodeAttributeBase {
    id: NodeId,
    name_space: NameSpace,
    merge_priority: MergePriority,
    expose_static: Option<bool>,
}

impl Parse for NodeAttributeBase {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        let name = node.attribute_of(NAME).unwrap();
        let id = store.id_by_name(&name);
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

#[derive(Debug, Clone)]
pub(super) struct NodeElementBase {
    pub(super) tool_tip: Option<String>,
    pub(super) description: Option<String>,
    pub(super) display_name: Option<String>,
    pub(super) visibility: Visibility,
    pub(super) docu_url: Option<String>,
    pub(super) is_deprecated: bool,
    pub(super) event_id: Option<u64>,
    pub(super) p_is_implemented: Option<NodeId>,
    pub(super) p_is_available: Option<NodeId>,
    pub(super) p_is_locked: Option<NodeId>,
    pub(super) p_block_polling: Option<NodeId>,
    pub(super) imposed_access_mode: AccessMode,
    pub(super) p_errors: Vec<NodeId>,
    pub(super) p_alias: Option<NodeId>,
    pub(super) p_cast_alias: Option<NodeId>,
}

impl Parse for NodeElementBase {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        // Ignore Extension element.
        node.parse_if::<String>(EXTENSION, store);

        let tool_tip = node.parse_if(TOOL_TIP, store);
        let description = node.parse_if(DESCRIPTION, store);
        let display_name = node.parse_if(DISPLAY_NAME, store);
        let visibility = node.parse_if(VISIBILITY, store).unwrap_or_default();
        let docu_url = node.parse_if(DOCU_URL, store);
        let is_deprecated = node.parse_if(IS_DEPRECATED, store).unwrap_or_default();
        let event_id = node
            .next_if(EVENT_ID)
            .map(|n| u64::from_str_radix(n.text(), 16).unwrap());
        let p_is_implemented = node.parse_if(P_IS_IMPLEMENTED, store);
        let p_is_available = node.parse_if(P_IS_AVAILABLE, store);
        let p_is_locked = node.parse_if(P_IS_LOCKED, store);
        let p_block_polling = node.parse_if(P_BLOCK_POLLING, store);
        let imposed_access_mode = node
            .parse_if(IMPOSED_ACCESS_MODE, store)
            .unwrap_or(AccessMode::RW);
        let p_errors = node.parse_while(P_ERROR, store);
        let p_alias = node.parse_if(P_ALIAS, store);
        let p_cast_alias = node.parse_if(P_CAST_ALIAS, store);

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
