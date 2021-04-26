use crate::{
    formula::{parse, Formula},
    store::{NodeStore, ValueStore},
};

use super::{xml, Parse};

impl Parse for Formula {
    fn parse<T, U>(node: &mut xml::Node, _: &mut T, _: &mut U) -> Self
    where
        T: NodeStore,
        U: ValueStore,
    {
        let text = node.next_text().unwrap();
        let expr = parse(text);
        Formula { expr }
    }
}
