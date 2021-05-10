//! This module contains types that is the main entry types of the `Cameleon`.
use super::{payload::PayloadSender, ControlResult, StreamResult};

/// This trait provides I/O s to the device's memory.
pub trait DeviceControl {
    /// Open the handle.
    fn open(&mut self) -> ControlResult<()>;

    /// Close the handle.
    fn close(&mut self) -> ControlResult<()>;

    /// Return `true` if device is already opened.
    fn is_opened(&self) -> bool;

    /// Read data from the device's memory.
    ///
    /// Read length is same as `buf.len()`.
    fn read_mem(&mut self, address: u64, buf: &mut [u8]) -> ControlResult<()>;

    /// Write data to the device's memory.
    fn write_mem(&mut self, address: u64, data: &[u8]) -> ControlResult<()>;

    /// Return `GenICam` xml string.
    fn gen_api(&mut self) -> ControlResult<String>;

    /// Enable streaming.
    fn enable_streaming(&mut self) -> ControlResult<()>;

    /// Disable streaming.
    fn disable_streaming(&mut self) -> ControlResult<()>;
}

/// This trait provides streaming capability.
pub trait PayloadStream {
    /// Open the handle.
    fn open(&mut self) -> StreamResult<()>;

    /// Close the handle.
    fn close(&mut self) -> StreamResult<()>;

    /// Start streaming.
    fn run_streaming_loop(&mut self, sender: PayloadSender) -> StreamResult<()>;

    /// Stop streaming.
    fn stop_streaming_loop(&mut self) -> StreamResult<()>;

    /// Return `true` if streaming loop is running.
    fn is_loop_running(&self) -> bool;
}
