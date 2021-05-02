use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    StringRegNode,
};

use super::{xml, Parse};

impl Parse for StringRegNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `StringRegNode`");
        debug_assert!(node.tag_name() == "StringReg");

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let register_base = node.parse(node_builder, value_builder, cache_builder);

        let node = Self {
            attr_base,
            register_base,
        };
        node.register_base
            .store_invalidators(node.attr_base.id, cache_builder);
        node
    }
}
