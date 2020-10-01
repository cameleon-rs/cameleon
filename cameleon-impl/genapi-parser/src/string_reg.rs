use super::{node_base::*, register_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct StringRegNode {
    attr_base: NodeAttributeBase,
    register_base: RegisterBase,
}

impl StringRegNode {
    pub fn node_base(&self) -> NodeBase {
        let elem_base = &self.register_base.elem_base;
        NodeBase::new(&self.attr_base, elem_base)
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }
}

impl Parse for StringRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "StringReg");

        let attr_base = node.parse();
        let register_base = node.parse();

        Self {
            attr_base,
            register_base,
        }
    }
}
