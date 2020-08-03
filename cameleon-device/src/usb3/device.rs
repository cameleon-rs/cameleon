use std::fmt;

use semver::Version;

use super::channel::*;
use super::Result;

pub(super) type RusbDevHandle = rusb::DeviceHandle<rusb::GlobalContext>;
pub(super) type RusbDevice = rusb::Device<rusb::GlobalContext>;

pub struct Device {
    device: RusbDevice,

    ctrl_iface_info: ControlIfaceInfo,
    event_iface_info: Option<ReceiveIfaceInfo>,
    stream_iface_info: Option<ReceiveIfaceInfo>,

    device_info: DeviceInfo,
}

impl Device {
    pub fn control_channel(&self) -> Result<ControlChannel> {
        let device_handle = self.device.open()?;

        Ok(ControlChannel::new(
            device_handle,
            self.ctrl_iface_info.clone(),
        ))
    }

    pub fn event_channel(&self) -> Result<Option<ReceiveChannel>> {
        let device_handle = self.device.open()?;

        match &self.event_iface_info {
            Some(iface_info) => Ok(Some(ReceiveChannel::new(device_handle, iface_info.clone()))),
            None => Ok(None),
        }
    }

    pub fn stream_channel(&self) -> Result<Option<ReceiveChannel>> {
        let device_handle = self.device.open()?;

        match &self.stream_iface_info {
            Some(iface_info) => Ok(Some(ReceiveChannel::new(device_handle, iface_info.clone()))),
            None => Ok(None),
        }
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    pub(super) fn new(
        device: RusbDevice,
        ctrl_iface_info: ControlIfaceInfo,
        event_iface_info: Option<ReceiveIfaceInfo>,
        stream_iface_info: Option<ReceiveIfaceInfo>,
        device_info: DeviceInfo,
    ) -> Self {
        let device = Self {
            device,
            ctrl_iface_info,
            event_iface_info,
            stream_iface_info,
            device_info,
        };

        log::info! {"{}: create device", device.log_name()};

        device
    }

    //TODO: We need logger.
    fn log_name(&self) -> String {
        format!(
            "{}-{}-{}",
            self.device_info.vendor_name,
            self.device_info.model_name,
            self.device_info.serial_number,
        )
    }
}

/// Device information in class-specific device descriptor.
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    /// GenCP version the device provides.
    pub gen_cp_version: Version,

    /// USB3-Vision version the device provides.
    pub u3v_version: Version,

    /// Device GUID consists of 12 characters.
    /// First 4 characters are vendor ID and last 8 characters are unique id assigned by a vendor.
    pub guid: String,

    /// Manufacture name of the device.
    pub vendor_name: String,

    /// Model name of the device.
    pub model_name: String,

    /// A human readable name referring to multiple models of a single manufacture.
    pub family_name: Option<String>,

    /// Manufacture specific device version.
    /// An application can't make any assumptions of this version.
    pub device_version: String,

    /// Manufacture specific information.
    /// This field is optional.
    pub manufacture_info: String,

    /// Serial number of the device.
    pub serial_number: String,

    /// User defined name.
    /// This field is optional.
    pub user_defined_name: Option<String>,

    /// Bus speed supported by the device.
    pub supported_speed: SupportedSpeed,
}

/// Bus speed supported by each USB device.
#[derive(Clone, Debug)]
pub enum SupportedSpeed {
    /// USB 1.0/Low-Speed: 1.5 Mbps
    LowSpeed,

    /// USB 1.1/Full-Speed: 12 Mbps
    FullSpeed,

    /// USB 2.0/Hi-Speed: 480 Mbps
    HighSpeed,

    /// USB 3.0/SuperSpeed: 5 Gbps
    SuperSpeed,

    /// USB 3.1/SuperSpeedPlus: 10 Gbps
    SuperSpeedPlus,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "### Device Information ###")?;
        writeln!(f, "U3V Version: {}", self.u3v_version)?;
        writeln!(f, "GUID: {}", self.guid)?;
        writeln!(f, "Vendor Name: {}", self.vendor_name)?;
        writeln!(f, "Model Name: {}", self.model_name)?;
        if let Some(family_name) = &self.family_name {
            writeln!(f, "Family Name: {}", family_name)?;
        } else {
            writeln!(f, "Family Name: N/A")?;
        }
        writeln!(f, "Manufacture Information: {}", self.manufacture_info)?;
        writeln!(f, "Serial Number: {}", self.serial_number)?;
        if let Some(user_defined_name) = &self.user_defined_name {
            writeln!(f, "User Defined Name: {}", user_defined_name)?;
        } else {
            writeln!(f, "User Defined Name: N/A")?;
        }
        write!(f, "Supported Speed: {:?}", self.supported_speed)?;
        Ok(())
    }
}
