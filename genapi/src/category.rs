/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    interface::{ICategory, INode},
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    store::{NodeId, NodeStore},
};

#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub(crate) attr_base: NodeAttributeBase,
    pub(crate) elem_base: NodeElementBase,

    pub(crate) p_features: Vec<NodeId>,
}

impl CategoryNode {
    #[must_use]
    pub fn p_features(&self) -> &[NodeId] {
        &self.p_features
    }
}

impl INode for CategoryNode {
    fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    fn streamable(&self) -> bool {
        false
    }
}

impl ICategory for CategoryNode {
    fn nodes(&self, _: &impl NodeStore) -> &[NodeId] {
        self.p_features()
    }
}
