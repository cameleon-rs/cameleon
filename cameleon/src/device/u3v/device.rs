use cameleon_device::u3v;

use crate::device::{DeviceError, DeviceResult};

use super::{control_handle::ControlHandle, register_map::Abrm};

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
/// use cameleon::device::u3v::enumerate_devices;
///
/// // Enumerate devices connected to the host.
/// let mut devices = enumerate_devices().unwrap();
///
/// // If no device is found, return.
/// if devices.is_empty() {
///     return;
/// }
///
/// let mut device = devices.pop().unwrap();
///
/// // Open the device to obtain exclusive access to the device.
/// device.open().unwrap();
///
/// // Get control handle to manipulate the device.
/// let handle = device.control_handle().unwrap();
/// ```
pub struct Device {
    device: u3v::Device,

    // TODO: Stream and event handles.
    ctrl_handle: ControlHandle,
}

impl Device {
    /// Open the device to obtain access.
    pub fn open(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        self.ctrl_handle.open()?;
        Ok(())
    }

    /// Close the device.
    ///
    /// After closing the device all handles of the device can't communicate with device.  
    /// This method is automatically called inside `Drop::drop`.
    pub fn close(&mut self) -> DeviceResult<()> {
        self.ctrl_handle.close()
    }

    /// Return control handle of the device.
    pub fn control_handle(&self) -> DeviceResult<&ControlHandle> {
        self.assert_open()?;

        Ok(&self.ctrl_handle)
    }

    /// Return `true` if device is opened.
    pub fn is_opened(&self) -> bool {
        self.ctrl_handle.is_opened()
    }

    /// Return Technology Agnostic Boot Register Map of the device.
    pub fn abrm(&self) -> DeviceResult<Abrm> {
        self.assert_open()?;

        self.ctrl_handle.abrm()
    }

    /// Basic information of the device. No need to call [`Device::open`] to obtain the
    /// information.
    ///
    /// # Examples
    /// ```no_run
    /// use cameleon::device::u3v::enumerate_devices;
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
    pub fn device_info(&self) -> &DeviceInfo {
        self.device.device_info()
    }

    /// Construct device from `cameleon_device::u3v::Device`.
    ///
    /// In normal use case, use [`enumerate_devices`] to construct devices.
    pub fn new(device: u3v::Device) -> DeviceResult<Self> {
        let ctrl_handle = ControlHandle::new(&device)?;

        Ok(Self {
            device,
            ctrl_handle,
        })
    }

    fn assert_open(&self) -> DeviceResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(DeviceError::NotOpened)
        }
    }
}

/// Enumerate all U3V compatible devices connected to the host.
///
/// # Examples
/// ```no_run
/// use cameleon::device::u3v::enumerate_devices;
///
/// // Enumerate devices connected to the host.
/// let mut devices = enumerate_devices().unwrap();
/// ```
pub fn enumerate_devices() -> DeviceResult<Vec<Device>> {
    let devices = u3v::enumerate_devices()?;

    Ok(devices
        .into_iter()
        .map(Device::new)
        .filter_map(|d| d.ok())
        .collect())
}

impl Drop for Device {
    fn drop(&mut self) {
        // TODO: log.
        let _ = self.close();
    }
}
