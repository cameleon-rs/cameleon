#[cfg(test)]
pub(super) mod tests {
    use super::super::{xml, Parse};
    use crate::store::{DefaultCacheStore, DefaultNodeStore, DefaultValueStore};

    pub(in super::super) fn parse_default<T: Parse>(
        xml: &str,
    ) -> (T, DefaultNodeStore, DefaultValueStore, DefaultCacheStore) {
        let document = xml::Document::from_str(xml).unwrap();
        let mut node_builder = DefaultNodeStore::new();
        let mut value_builder = DefaultValueStore::new();
        let mut cache_builder = DefaultCacheStore::new();

        (
            document
                .root_node()
                .parse(&mut node_builder, &mut value_builder, &mut cache_builder),
            node_builder,
            value_builder,
            cache_builder,
        )
    }
}
