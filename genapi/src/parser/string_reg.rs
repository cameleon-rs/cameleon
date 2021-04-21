use crate::{store::NodeStore, StringRegNode};

use super::{xml, Parse};

impl Parse for StringRegNode {
    fn parse<T>(node: &mut xml::Node, store: &mut T) -> Self
    where
        T: NodeStore,
    {
        debug_assert!(node.tag_name() == "StringReg");

        let attr_base = node.parse(store);
        let register_base = node.parse(store);

        Self {
            attr_base,
            register_base,
        }
    }
}
