/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![recursion_limit = "1024"]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation
)]

#[cfg(feature = "libusb")]
pub mod u3v;

mod pixel_format;

pub use pixel_format::PixelFormat;
