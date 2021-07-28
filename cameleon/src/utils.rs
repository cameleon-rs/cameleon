/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

macro_rules! unwrap_or_log {
    ($expr:expr) => {{
        use tracing::error;
        match $expr {
            Ok(v) => v,
            Err(error) => {
                error!(?error);
                return Err(error.into());
            }
        }
    }};
}
