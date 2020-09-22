use super::elem_type::*;

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
            self.elem.$name.as_ref().map(|elem| elem.as_str())
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

    optional_string_elem_getter! {tool_tip}

    pub fn display_name(&self) -> &str {
        if let Some(display_name) = self.elem.display_name.as_ref() {
            &display_name
        } else {
            self.name()
        }
    }

    pub fn visibility(&self) -> Visibility {
        self.elem.visibility
    }

    optional_string_elem_getter! {docu_url}

    pub fn is_deprecated(&self) -> bool {
        self.elem.is_deprecated
    }

    optional_string_elem_getter! {event_id}
    optional_string_elem_getter! {p_is_implemented}
    optional_string_elem_getter! {p_is_available}
    optional_string_elem_getter! {p_is_locked}
    optional_string_elem_getter! {p_block_polling}

    pub fn imposed_access_mode(&self) -> AccessMode {
        self.elem.imposed_access_mode
    }

    optional_string_elem_getter! {p_error}
    optional_string_elem_getter! {p_alias}
    optional_string_elem_getter! {p_cast_alias}
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub(super) struct NodeAttributeBase {
    name: String,

    name_space: NameSpace,

    merge_priority: MergePriority,

    expose_static: Option<bool>,
}

impl NodeAttributeBase {}

#[derive(Debug, Clone, PartialEq, Eq)]
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
