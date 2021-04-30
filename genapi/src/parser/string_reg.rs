use crate::{
    store::{WritableNodeStore, ValueStore},
    StringRegNode,
};

use super::{xml, Parse};

impl Parse for StringRegNode {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        debug_assert!(node.tag_name() == "StringReg");

        let attr_base = node.parse(node_store, value_store);
        let register_base = node.parse(node_store, value_store);

        Self {
            attr_base,
            register_base,
        }
    }
}
