use tracing::debug;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    elem_type::ImmOrPNode,
    PortNode,
};

use super::{
    elem_name::{CACHE_CHUNK_DATA, CHUNK_ID, PORT, P_CHUNK_ID, SWAP_ENDIANNESS},
    xml, Parse,
};

impl Parse for PortNode {
    #[tracing::instrument(level = "trace", skip(node_builder, value_builder, cache_builder))]
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        debug!("start parsing `PortNode`");
        debug_assert_eq!(node.tag_name(), PORT);

        let attr_base = node.parse(node_builder, value_builder, cache_builder);
        let elem_base = node.parse(node_builder, value_builder, cache_builder);

        let chunk_id = node.next_if(CHUNK_ID).map_or_else(
            || {
                node.next_if(P_CHUNK_ID)
                    .map(|next_node| ImmOrPNode::PNode(node_builder.get_or_intern(next_node.text())))
            },
            |next_node| {
                Some(ImmOrPNode::Imm(
                    u64::from_str_radix(next_node.text(), 16).unwrap(),
                ))
            },
        );
        let swap_endianness = node
            .parse_if(SWAP_ENDIANNESS, node_builder, value_builder, cache_builder)
            .unwrap_or_default();
        let cache_chunk_data = node
            .parse_if(CACHE_CHUNK_DATA, node_builder, value_builder, cache_builder)
            .unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            chunk_id,
            swap_endianness,
            cache_chunk_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::utils::tests::parse_default, *};

    #[test]
    fn test_port_node_with_imm() {
        let xml = r#"
            <Port Name="TestNode">
                <ChunkID>Fd3219</ChunkID>
                <SwapEndianess>Yes</SwapEndianess>
            <Port>
            "#;

        let (node, ..): (PortNode, _, _, _) = parse_default(xml);
        assert_eq!(node.chunk_id().unwrap(), &ImmOrPNode::Imm(0x00FD_3219));
        assert_eq!(node.swap_endianness(), true);
    }

    #[test]
    fn test_port_node_with_p_node() {
        let xml = r#"
            <Port Name="TestNode">
                <pChunkID>Fd3219</pChunkID>
            <Port>
            "#;

        let (node, mut node_builder, ..): (PortNode, _, _, _) = parse_default(xml);
        assert_eq!(
            node.chunk_id().unwrap(),
            &ImmOrPNode::PNode(node_builder.get_or_intern("Fd3219"))
        );
    }

    #[test]
    fn test_port_node_without_chunk_id() {
        let xml = r#"
            <Port Name="TestNode">
                <CacheChunkData>Yes</CacheChunkData>
            <Port>
            "#;

        let (node, ..): (PortNode, _, _, _) = parse_default(xml);
        assert_eq!(node.chunk_id(), None);
        assert_eq!(node.cache_chunk_data(), true);
    }
}
