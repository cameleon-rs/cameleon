/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use syn::{Error, Result};

pub(super) fn modify_visibility(vis: &syn::Visibility) -> Result<syn::Visibility> {
    use syn::Visibility::{Crate, Inherited, Public, Restricted};
    match vis {
        Public(_) | Crate(_) => Ok(vis.clone()),
        Inherited => Ok(syn::parse_str("pub(super)").unwrap()),
        Restricted(restricted) => {
            let original = restricted.path.get_ident().unwrap();
            if original == "crate" {
                Ok(syn::parse_str("pub(crate)").unwrap())
            } else if original == "super" {
                Ok(syn::parse_str("pub(in super::super)").unwrap())
            } else if original == "self" {
                Ok(syn::parse_str("pub(super)").unwrap())
            } else {
                Err(Error::new_spanned(vis, "pub(in ...) can't be used"))
            }
        }
    }
}
