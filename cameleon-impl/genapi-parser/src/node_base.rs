use super::{elem_type::*, xml};

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

    optional_string_elem_getter! {tool_tip}
    optional_string_elem_getter! {docu_url}
    optional_string_elem_getter! {event_id}
    optional_string_elem_getter! {p_is_implemented}
    optional_string_elem_getter! {p_is_available}
    optional_string_elem_getter! {p_is_locked}
    optional_string_elem_getter! {p_block_polling}
    optional_string_elem_getter! {p_error}
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

impl NodeAttributeBase {
    pub(super) fn parse(node: &xml::Node) -> Self {
        let name = node.attribute_of("Name").unwrap().into();

        let name_space = node
            .attribute_of("NameSpace")
            .map(|text| text.into())
            .unwrap_or_default();

        let merge_priority = node
            .attribute_of("MergePriority")
            .map(|text| text.into())
            .unwrap_or_default();

        let expose_static = node
            .attribute_of("ExposeStatic")
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
    tool_tip: Option<String>,

    display_name: Option<String>,

    visibility: Visibility,

    docu_url: Option<String>,

    is_deprecated: bool,

    event_id: Option<String>,

    p_is_implemented: Option<String>,

    p_is_available: Option<String>,

    p_is_locked: Option<String>,

    p_block_polling: Option<String>,

    imposed_access_mode: AccessMode,

    p_error: Option<String>,

    p_alias: Option<String>,

    p_cast_alias: Option<String>,
}

impl NodeElementBase {
    pub(super) fn parse(node: &mut xml::Node) -> Self {
        node.next_if("Extension");

        let tool_tip = node.next_text_if("ToolTip").map(Into::into);

        let display_name = node.next_text_if("DisplayName").map(Into::into);

        let visibility = node
            .next_text_if("Visibility")
            .map(|text| text.into())
            .unwrap_or_default();

        let docu_url = node.next_text_if("DocuURL").map(Into::into);

        let is_deprecated = node
            .next_text_if("IsDeprecated")
            .map(|text| convert_to_bool(&text))
            .unwrap_or(false);

        let event_id = node.next_text_if("EventID").map(Into::into);

        let p_is_implemented = node.next_text_if("pIsImplemented").map(Into::into);

        let p_is_available = node.next_text_if("pIsAvailable").map(Into::into);

        let p_is_locked = node.next_text_if("pIsLocked").map(Into::into);

        let p_block_polling = node.next_text_if("pBlockPolling").map(Into::into);

        let imposed_access_mode = node
            .next_text_if("ImposedAccessMode")
            .map(|text| text.into())
            .unwrap_or(AccessMode::RW);

        let p_error = node.next_text_if("pError").map(Into::into);

        let p_alias = node.next_text_if("pAlias").map(Into::into);

        let p_cast_alias = node.next_text_if("pCastAlias").map(Into::into);

        Self {
            tool_tip,
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
            p_error,
            p_alias,
            p_cast_alias,
        }
    }
}
