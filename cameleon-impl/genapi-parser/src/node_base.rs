use super::{elem_name::*, elem_type::*, xml, Parse};

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
        pub fn $name(&self) -> Option<&str> {
            self.elem.$name.as_deref()
        }
    };
}

impl<'a> NodeBase<'a> {
    pub(super) fn new(attr: &'a NodeAttributeBase, elem: &'a NodeElementBase) -> Self {
        Self { attr, elem }
    }

    pub fn name(&self) -> &str {
        &self.attr.name
    }

    pub fn name_space(&self) -> NameSpace {
        self.attr.name_space
    }

    pub fn merge_priority(&self) -> MergePriority {
        self.attr.merge_priority
    }

    pub fn expose_static(&self) -> Option<bool> {
        self.attr.expose_static
    }

    pub fn display_name(&self) -> &str {
        if let Some(ref display_name) = self.elem.display_name {
            display_name
        } else {
            self.name()
        }
    }

    pub fn visibility(&self) -> Visibility {
        self.elem.visibility
    }

    pub fn is_deprecated(&self) -> bool {
        self.elem.is_deprecated
    }

    pub fn imposed_access_mode(&self) -> AccessMode {
        self.elem.imposed_access_mode
    }

    pub fn p_errors(&self) -> &[String] {
        &self.elem.p_errors
    }

    optional_string_elem_getter! {description}
    optional_string_elem_getter! {tool_tip}
    optional_string_elem_getter! {docu_url}
    optional_string_elem_getter! {event_id}
    optional_string_elem_getter! {p_is_implemented}
    optional_string_elem_getter! {p_is_available}
    optional_string_elem_getter! {p_is_locked}
    optional_string_elem_getter! {p_block_polling}
    optional_string_elem_getter! {p_alias}
    optional_string_elem_getter! {p_cast_alias}
}

#[derive(Debug, Clone)]
pub(super) struct NodeAttributeBase {
    name: String,
    name_space: NameSpace,
    merge_priority: MergePriority,
    expose_static: Option<bool>,
}

impl Parse for NodeAttributeBase {
    fn parse(node: &mut xml::Node) -> Self {
        let name = node.attribute_of(NAME).unwrap().into();
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
            name,
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
    pub(super) event_id: Option<String>,
    pub(super) p_is_implemented: Option<String>,
    pub(super) p_is_available: Option<String>,
    pub(super) p_is_locked: Option<String>,
    pub(super) p_block_polling: Option<String>,
    pub(super) imposed_access_mode: AccessMode,
    pub(super) p_errors: Vec<String>,
    pub(super) p_alias: Option<String>,
    pub(super) p_cast_alias: Option<String>,
}

impl Parse for NodeElementBase {
    fn parse(node: &mut xml::Node) -> Self {
        // Ignore Extension element.
        node.parse_if::<String>(EXTENSION);

        let tool_tip = node.parse_if(TOOL_TIP);
        let description = node.parse_if(DESCRIPTION);
        let display_name = node.parse_if(DISPLAY_NAME);
        let visibility = node.parse_if(VISIBILITY).unwrap_or_default();
        let docu_url = node.parse_if(DOCU_URL);
        let is_deprecated = node.parse_if(IS_DEPRECATED).unwrap_or_default();
        let event_id = node.parse_if(EVENT_ID);
        let p_is_implemented = node.parse_if(P_IS_IMPLEMENTED);
        let p_is_available = node.parse_if(P_IS_AVAILABLE);
        let p_is_locked = node.parse_if(P_IS_LOCKED);
        let p_block_polling = node.parse_if(P_BLOCK_POLLING);
        let imposed_access_mode = node.parse_if(IMPOSED_ACCESS_MODE).unwrap_or(AccessMode::RW);
        let p_errors = node.parse_while(P_ERROR);
        let p_alias = node.parse_if(P_ALIAS);
        let p_cast_alias = node.parse_if(P_CAST_ALIAS);

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
