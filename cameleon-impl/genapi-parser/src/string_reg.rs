use super::{node_base::*, register_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct StringRegNode {
    register_base: RegisterBase,
}

impl StringRegNode {
    pub fn node_base(&self) -> NodeBase {
        self.register_base.node_base()
    }

    pub fn register_base(&self) -> &RegisterBase {
        &self.register_base
    }
}

impl Parse for StringRegNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert!(node.tag_name() == "StringReg");

        let register_base = node.parse();

        Self { register_base }
    }
}
