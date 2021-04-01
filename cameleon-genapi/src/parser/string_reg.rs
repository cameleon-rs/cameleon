use super::{
    node_base::{NodeAttributeBase, NodeBase},
    node_store::NodeStore,
    register_base::RegisterBase,
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct StringRegNode {
    attr_base: NodeAttributeBase,
    register_base: RegisterBase,
}

impl StringRegNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    #[must_use]
    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }
}

impl Parse for StringRegNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert!(node.tag_name() == "StringReg");

        let attr_base = node.parse(store);
        let register_base = node.parse(store);

        Self {
            attr_base,
            register_base,
        }
    }
}
