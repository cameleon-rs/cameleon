use crate::{
    formula::{parse, Expr, Formula},
    store::{NodeStore, ValueStore},
};

use super::{xml, Parse};

impl Parse for Formula {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        let expr = node.parse(node_store, value_store);
        Formula { expr }
    }
}

impl Parse for Expr {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        let text = node.next_text().unwrap();
        parse(text)
    }
}
