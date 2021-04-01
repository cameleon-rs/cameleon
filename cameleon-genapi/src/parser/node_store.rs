use std::convert::TryInto;

use string_interner::{StringInterner, Symbol};

use super::register_description::NodeData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(u32);

impl Symbol for NodeId {
    fn try_from_usize(index: usize) -> Option<Self> {
        let val: u32 = index.try_into().ok()?;
        Some(Self(val))
    }

    fn to_usize(self) -> usize {
        self.0 as usize
    }
}

pub struct NodeStore {
    interner: StringInterner<NodeId>,
    store: Vec<Option<NodeData>>,
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
            store: Vec::new(),
        }
    }

    pub fn id_by_name(&mut self, s: impl AsRef<str>) -> NodeId {
        self.interner.get_or_intern(s)
    }

    pub fn store_node(&mut self, id: NodeId, data: NodeData) {
        let id = id.to_usize();
        if self.store.len() < id {
            self.store.resize(id, None)
        }
        debug_assert!(self.store[id].is_none());
        self.store[id] = Some(data);
    }

    pub fn node(&self, id: NodeId) -> &NodeData {
        self.node_opt(id).unwrap()
    }

    pub fn node_opt(&self, id: NodeId) -> Option<&NodeData> {
        self.store.get(id.to_usize())?.as_ref()
    }
}

impl Default for NodeStore {
    fn default() -> Self {
        Self::new()
    }
}
