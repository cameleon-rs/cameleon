pub mod device_builder;

pub(super) mod fake_protocol;

mod control_module;
mod device;
mod device_pool;
mod event_module;
mod interface;
mod memory;
mod signal;
mod stream_module;

use thiserror::Error;
