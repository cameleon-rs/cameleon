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

pub mod gige;

#[cfg(feature = "libusb")]
pub mod u3v;

//// TODO: finish implementation.
//mod emulator;

mod pixel_format;

pub use pixel_format::PixelFormat;

pub enum Endianness {
    BE,
    LE,
}

pub enum CharacterEncoding {
    Utf8,
    Ascii,
}

/// Represent file type of `GenICam` XML file on the device's memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenICamFileType {
    /// This is the “normal” `GenICam` device XML containing all device features.
    DeviceXml,
    /// This is optional XML-file that contains only the chunkdata related nodes.
    BufferXml,
}

/// Represents `CompressionType` of `GenICam` XML file on the device's memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// Uncompressed `GenICam` XML file.
    Uncompressed,
    /// ZIP containing a single `GenICam` XML file.
    Zip,
}
