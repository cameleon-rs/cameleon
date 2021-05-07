//! TODO: TBW
use super::DeviceResult;

/// This trait provides I/O s to the device's memory.
pub trait DeviceControl {
    /// Open the handle.
    fn open(&mut self) -> DeviceResult<()>;

    /// Close the handle.
    fn close(&mut self) -> DeviceResult<()>;

    /// Return `true` if device is already opened.
    fn is_opened(&self) -> bool;

    /// Read data from the device's memory.
    ///
    /// Read length is same as `buf.len()`.
    fn read_mem(&mut self, address: u64, buf: &mut [u8]) -> DeviceResult<()>;

    /// Write data to the device's memory.
    fn write_mem(&mut self, address: u64, data: &[u8]) -> DeviceResult<()>;

    /// Return `GenICam` xml string.
    fn gen_api(&mut self) -> DeviceResult<String>;

    /// Enable streaming.
    fn enable_streaming(&mut self) -> DeviceResult<()>;

    /// Disable streaming.
    fn disable_streaming(&mut self) -> DeviceResult<()>;
}

/// TODO:
pub trait ImageStream {}
