use crate::{elem_type::ImmOrPNode, node_store::NodeStore, PortNode};

use super::{
    elem_name::{CACHE_CHUNK_DATA, CHUNK_ID, PORT, P_CHUNK_ID, P_INVALIDATOR, SWAP_ENDIANNESS},
    xml, Parse,
};

impl Parse for PortNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), PORT);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let chunk_id = node.next_if(CHUNK_ID).map_or_else(
            || {
                node.next_if(P_CHUNK_ID)
                    .map(|next_node| ImmOrPNode::PNode(store.id_by_name(next_node.text())))
            },
            |next_node| {
                Some(ImmOrPNode::Imm(
                    u64::from_str_radix(next_node.text(), 16).unwrap(),
                ))
            },
        );
        let swap_endianness = node.parse_if(SWAP_ENDIANNESS, store).unwrap_or_default();
        let cache_chunk_data = node.parse_if(CACHE_CHUNK_DATA, store).unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            chunk_id,
            swap_endianness,
            cache_chunk_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_node_with_imm() {
        let xml = r#"
            <Port Name="TestNode">
                <ChunkID>Fd3219</ChunkID>
                <SwapEndianess>Yes</SwapEndianess>
            <Port>
            "#;

        let mut store = NodeStore::new();
        let node: PortNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

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

        let mut store = NodeStore::new();
        let node: PortNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(
            node.chunk_id().unwrap(),
            &ImmOrPNode::PNode(store.id_by_name("Fd3219"))
        );
    }

    #[test]
    fn test_port_node_without_chunk_id() {
        let xml = r#"
            <Port Name="TestNode">
                <CacheChunkData>Yes</CacheChunkData>
            <Port>
            "#;

        let mut store = NodeStore::new();
        let node: PortNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.chunk_id(), None);
        assert_eq!(node.cache_chunk_data(), true);
    }
}
