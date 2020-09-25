use std::convert::TryInto;

use super::{elem_type::*, verifier::*, xml, GenApiError, GenApiResult, Span};

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
        pub fn $name(&self) -> Option<Span<&str>> {
            match self.elem.$name {
                Some(ref elem) => Some(elem.span(elem.as_str())),
                _ => None
            }
        }
    };
}

impl<'a> NodeBase<'a> {
    pub(super) fn new(attr: &'a NodeAttributeBase, elem: &'a NodeElementBase) -> Self {
        Self { attr, elem }
    }

    pub fn name(&self) -> Span<&str> {
        self.attr.name.span(self.attr.name.as_str())
    }

    pub fn name_space(&self) -> Span<NameSpace> {
        self.attr.name_space
    }

    pub fn merge_priority(&self) -> Span<MergePriority> {
        self.attr.merge_priority
    }

    pub fn expose_static(&self) -> Option<Span<bool>> {
        self.attr.expose_static
    }

    pub fn display_name(&self) -> Span<&str> {
        if let Some(ref display_name) = self.elem.display_name {
            display_name.span(display_name.as_str())
        } else {
            self.name()
        }
    }

    pub fn visibility(&self) -> Span<Visibility> {
        self.elem.visibility
    }

    pub fn is_deprecated(&self) -> Span<bool> {
        self.elem.is_deprecated
    }

    pub fn imposed_access_mode(&self) -> Span<AccessMode> {
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
    name: Span<String>,

    name_space: Span<NameSpace>,

    merge_priority: Span<MergePriority>,

    expose_static: Option<Span<bool>>,
}

impl NodeAttributeBase {
    pub(super) fn parse(node: &mut Span<xml::Node>) -> GenApiResult<Self> {
        let node_range = node.range();
        let name = node.next_attribute_if("Name").ok_or_else(|| {
            GenApiError::RequiredFieldMissing(Span::from_range("Name", node_range))
        })?;
        verify_node_name(name.value())?;
        let name: Span<String> = name.value().map(Into::into);

        let name_space: Span<NameSpace> = node
            .next_attribute_if("NameSpace")
            .map(|attr| attr.value().try_into())
            .transpose()?
            .unwrap_or(node.span(Default::default()));

        let merge_priority: Span<MergePriority> = node
            .next_attribute_if("MergePriority")
            .map(|attr| attr.value().try_into())
            .transpose()?
            .unwrap_or(node.span(Default::default()));

        let expose_static = node
            .next_attribute_if("ExposeStatic")
            .map(|attr| attr.value().try_into())
            .transpose()?;

        Ok(Self {
            name,
            name_space,
            merge_priority,
            expose_static,
        })
    }
}

#[derive(Debug, Clone)]
pub(super) struct NodeElementBase {
    tool_tip: Option<Span<String>>,

    display_name: Option<Span<String>>,

    visibility: Span<Visibility>,

    docu_url: Option<Span<String>>,

    is_deprecated: Span<bool>,

    event_id: Option<Span<String>>,

    p_is_implemented: Option<Span<String>>,

    p_is_available: Option<Span<String>>,

    p_is_locked: Option<Span<String>>,

    p_block_polling: Option<Span<String>>,

    imposed_access_mode: Span<AccessMode>,

    p_error: Option<Span<String>>,

    p_alias: Option<Span<String>>,

    p_cast_alias: Option<Span<String>>,
}

impl NodeElementBase {
    pub(super) fn parse(node: &mut Span<xml::Node>) -> GenApiResult<Self> {
        node.next_child_elem_if("Extension");

        let tool_tip = node
            .next_child_elem_text_if("ToolTip")?
            .map(|text| text.map(Into::into));

        let display_name = node
            .next_child_elem_text_if("DisplayName")?
            .map(|text| text.map(Into::into));

        let visibility = node.next_child_elem_text_if("Visibility")?.map_or_else(
            || Ok(node.span(Visibility::default())),
            |text| text.try_into(),
        )?;

        let docu_url = node
            .next_child_elem_text_if("DocuURL")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_url_string(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let is_deprecated = node
            .next_child_elem_text_if("IsDeprecated")?
            .map_or_else(|| Ok(node.span(false)), |text| text.try_into())?;

        let event_id = node
            .next_child_elem_text_if("EventID")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_hex_string(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let p_is_implemented = node
            .next_child_elem_text_if("pIsImplemented")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let p_is_available = node
            .next_child_elem_text_if("pIsAvailable")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let p_is_locked = node
            .next_child_elem_text_if("pIsLocked")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let p_block_polling = node
            .next_child_elem_text_if("pBlockPolling")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let imposed_access_mode = node
            .next_child_elem_text_if("ImposedAccessMode")?
            .map_or_else(|| Ok(node.span(AccessMode::RW)), |text| text.try_into())?;

        let p_error = node
            .next_child_elem_text_if("pError")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let p_alias = node
            .next_child_elem_text_if("pAlias")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        let p_cast_alias = node
            .next_child_elem_text_if("pCastAlias")?
            .map::<GenApiResult<_>, _>(|text| {
                verify_node_name(text)?;
                Ok(text.map(Into::into))
            })
            .transpose()?;

        Ok(Self {
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
        })
    }
}
