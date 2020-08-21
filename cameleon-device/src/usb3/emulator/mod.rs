mod peripheral;

pub(super) use peripheral::fake_protocol::*;

pub use peripheral::device_builder::{BuilderError, BuilderResult, DeviceBuilder};
