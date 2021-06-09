/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    formula::{parse, Expr, Formula},
};

use super::{xml, Parse};

impl Parse for Formula {
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        let expr = node.parse(node_builder, value_builder, cache_builder);
        Formula { expr }
    }
}

impl Parse for Expr {
    fn parse(
        node: &mut xml::Node,
        _: &mut impl NodeStoreBuilder,
        _: &mut impl ValueStoreBuilder,
        _: &mut impl CacheStoreBuilder,
    ) -> Self {
        let text = node.next_text().unwrap();
        parse(&text.view())
    }
}
