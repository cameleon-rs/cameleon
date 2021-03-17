#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc
)]

pub mod memory;

pub use cameleon_impl_genapi_parser as genapi_parser;

#[doc(hidden)]
pub use byteorder;

#[doc(hidden)]
pub use semver;
