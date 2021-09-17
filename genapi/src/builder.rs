/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    parser,
    store::{
        CacheSink, DefaultCacheStore, DefaultNodeStore, DefaultValueStore, NodeData, NodeId,
        ValueData, ValueId,
    },
    RegisterDescription, ValueCtxt,
};

#[derive(Default)]
pub struct GenApiBuilder<T = DefaultNodeStore, U = DefaultValueStore, S = DefaultCacheStore> {
    node_store: T,
    value_store: U,
    cache_store: S,
}

pub type BuildResult<T, U, S> = parser::ParseResult<(RegisterDescription, T, ValueCtxt<U, S>)>;

impl<T, U, S> GenApiBuilder<T, U, S> {
    pub fn build(mut self, xml: &impl AsRef<str>) -> BuildResult<T::Store, U::Store, S::Store>
    where
        T: NodeStoreBuilder,
        U: ValueStoreBuilder,
        S: CacheStoreBuilder,
    {
        let reg_desc = parser::parse(
            xml,
            &mut self.node_store,
            &mut self.value_store,
            &mut self.cache_store,
        )?;

        Ok((
            reg_desc,
            self.node_store.build(),
            ValueCtxt::new(self.value_store.build(), self.cache_store.build()),
        ))
    }

    pub fn no_cache(self) -> GenApiBuilder<T, U, CacheSink> {
        GenApiBuilder {
            node_store: self.node_store,
            value_store: self.value_store,
            cache_store: CacheSink::default(),
        }
    }

    pub fn with_node_store<T2>(self, node_store: T2) -> GenApiBuilder<T2, U, S> {
        GenApiBuilder {
            node_store,
            value_store: self.value_store,
            cache_store: self.cache_store,
        }
    }

    pub fn with_value_store<U2>(self, value_store: U2) -> GenApiBuilder<T, U2, S> {
        GenApiBuilder {
            node_store: self.node_store,
            value_store,
            cache_store: self.cache_store,
        }
    }

    pub fn with_cache_store<S2>(self, cache_store: S2) -> GenApiBuilder<T, U, S2> {
        GenApiBuilder {
            node_store: self.node_store,
            value_store: self.value_store,
            cache_store,
        }
    }
}

pub trait NodeStoreBuilder {
    type Store;

    /// Build `NodeStore`.
    fn build(self) -> Self::Store;

    /// Store [`NodeData`].
    fn store_node(&mut self, nid: NodeId, data: NodeData);

    /// Intern the node name and return the corresponding [`NodeId`].
    fn get_or_intern<T>(&mut self, node_name: T) -> NodeId
    where
        T: AsRef<str>;

    /// Returns fresh id for each call.
    fn fresh_id(&mut self) -> u32;
}

pub trait ValueStoreBuilder {
    type Store;

    /// Build `ValueStore`.
    fn build(self) -> Self::Store;

    /// Store the `ValueData` and return the corresponding [`ValueId`].
    fn store<T, U>(&mut self, data: T) -> U
    where
        T: Into<ValueData>,
        U: From<ValueId>;
}

pub trait CacheStoreBuilder {
    type Store;

    /// Build `CacheStore`.
    fn build(self) -> Self::Store;

    /// Store invalidator and its target to be invalidated.
    fn store_invalidator(&mut self, invalidator: NodeId, target: NodeId);
}
