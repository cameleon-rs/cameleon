#![recursion_limit = "1024"]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation
)]

#[cfg(feature = "libusb")]
pub mod u3v;

//// TODO: finish implementation.
//mod emulator;

mod pixel_format;

pub use pixel_format::PixelFormat;
