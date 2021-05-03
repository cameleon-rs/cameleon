use super::{
    elem_type::{AccessMode, MergePriority, NameSpace, Visibility},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    utils::bool_from_id,
    Device, GenApiError, GenApiResult, ValueCtxt,
};

pub struct NodeBase<'a> {
    pub(crate) attr: &'a NodeAttributeBase,
    pub(crate) elem: &'a NodeElementBase,
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
    pub(crate) fn new(attr: &'a NodeAttributeBase, elem: &'a NodeElementBase) -> Self {
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
pub(crate) struct NodeAttributeBase {
    pub(crate) id: NodeId,
    pub(crate) name_space: NameSpace,
    pub(crate) merge_priority: MergePriority,
    pub(crate) expose_static: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeElementBase {
    pub(crate) tool_tip: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) visibility: Visibility,
    pub(crate) docu_url: Option<String>,
    pub(crate) is_deprecated: bool,
    pub(crate) event_id: Option<u64>,
    pub(crate) p_is_implemented: Option<NodeId>,
    pub(crate) p_is_available: Option<NodeId>,
    pub(crate) p_is_locked: Option<NodeId>,
    pub(crate) p_block_polling: Option<NodeId>,
    pub(crate) imposed_access_mode: AccessMode,
    pub(crate) p_errors: Vec<NodeId>,
    pub(crate) p_alias: Option<NodeId>,
    pub(crate) p_cast_alias: Option<NodeId>,
}

impl NodeElementBase {
    pub(super) fn is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        Ok(self.is_implemented(device, store, cx)?
            && self.is_available(device, store, cx)?
            && matches!(self.imposed_access_mode, AccessMode::RO | AccessMode::RW))
    }

    pub(super) fn is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        Ok(self.is_implemented(device, store, cx)?
            && self.is_available(device, store, cx)?
            && !self.is_locked(device, store, cx)?
            && matches!(self.imposed_access_mode, AccessMode::WO | AccessMode::RW))
    }

    pub(super) fn verify_is_readable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        if self.is_readable(device, store, cx)? {
            Ok(())
        } else {
            Err(GenApiError::access_denied(
                "the node is not readable".into(),
            ))
        }
    }

    pub(super) fn verify_is_writable<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<()> {
        if self.is_writable(device, store, cx)? {
            Ok(())
        } else {
            Err(GenApiError::access_denied(
                "the node is not writable".into(),
            ))
        }
    }

    fn is_locked<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.p_is_locked
            .map_or(Ok(false), |nid| bool_from_id(nid, device, store, cx))
    }

    fn is_implemented<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.p_is_implemented
            .map_or(Ok(true), |nid| bool_from_id(nid, device, store, cx))
    }

    fn is_available<T: ValueStore, U: CacheStore>(
        &self,
        device: &mut impl Device,
        store: &impl NodeStore,
        cx: &mut ValueCtxt<T, U>,
    ) -> GenApiResult<bool> {
        self.p_is_available
            .map_or(Ok(true), |nid| bool_from_id(nid, device, store, cx))
    }
}
