//! TODO: REMOVE this module.
use cameleon_device::u3v;

use crate::ControlResult;

use super::{control_handle::ControlHandle, stream_handle::StreamHandle};

/// Basic device information that is obtained without opening the device.
pub type DeviceInfo = u3v::DeviceInfo;

/// Represent U3V device.
///
/// This type has two roles.
/// 1. Do initial negotiation to obtain exclusive access to the device.
/// 2. Provide handles that actually have methods to manipulate device.
///
/// # Examples
///
/// ```no_run
/// use cameleon::u3v::enumerate_devices;
///
/// // Enumerate devices connected to the host.
/// let mut devices = enumerate_devices().unwrap();
///
/// // If no device is found, return.
/// if devices.is_empty() {
///     return;
/// }
///
/// let device = devices.pop().unwrap();
///
/// // Obtain and open the control handle to manipulate the device.
/// let handle = device.control_handle();
/// handle.open().unwrap();
/// ```
pub struct Device {
    device: u3v::Device,

    // TODO: Add Event handles.
    ctrl_handle: ControlHandle,
    strm_handle: Option<StreamHandle>,
}

impl Device {
    /// Return control handle of the device.
    #[must_use]
    pub fn control_handle(&self) -> &ControlHandle {
        &self.ctrl_handle
    }

    /// Return stream handle of the device.
    #[must_use]
    pub fn stream_handle(&self) -> Option<&StreamHandle> {
        self.strm_handle.as_ref()
    }

    /// Basic information of the device.
    ///
    /// # Examples
    /// ```no_run
    /// use cameleon::u3v::enumerate_devices;
    ///
    /// // Enumerate devices connected to the host.
    /// let mut devices = enumerate_devices().unwrap();
    ///
    /// // If no device is found, return.
    /// if devices.is_empty() {
    ///     return;
    /// }
    ///
    /// // Device information can be obtained without opening device.
    /// let device = devices.pop().unwrap();
    /// let device_info = device.device_info();
    ///
    /// ```
    #[must_use]
    pub fn device_info(&self) -> &DeviceInfo {
        self.device.device_info()
    }

    /// Construct device from `cameleon_device::u3v::Device`.
    ///
    /// In normal use case, use [`enumerate_devices`] to construct devices.
    pub fn new(device: u3v::Device) -> ControlResult<Self> {
        let ctrl_handle = ControlHandle::new(&device)?;
        let strm_handle = StreamHandle::new(&device)?;

        Ok(Self {
            device,
            ctrl_handle,
            strm_handle,
        })
    }
}

/// Enumerate all U3V compatible devices connected to the host.
///
/// # Examples
/// ```no_run
/// use cameleon::u3v::enumerate_devices;
///
/// // Enumerate devices connected to the host.
/// let mut devices = enumerate_devices().unwrap();
/// ```
pub fn enumerate_devices() -> ControlResult<Vec<Device>> {
    let devices = u3v::enumerate_devices()?;

    Ok(devices
        .into_iter()
        .map(Device::new)
        .filter_map(std::result::Result::ok)
        .collect())
}
