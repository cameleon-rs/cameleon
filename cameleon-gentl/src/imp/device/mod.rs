use std::{convert::TryFrom, sync::Mutex};

use crate::{GenTlError, GenTlResult};

pub(crate) mod u3v;

use crate::imp::port::*;

mod u3v_genapi;

/// The current accessibility of the device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeviceAccessStatus {
    /// The current availability of the device is unknown.
    Unknown = 0,

    /// The device is available to be opened for Read/Write access but it is currently not opened.
    ReadWrite = 1,

    /// The device is available to be opened for Read access but is currently not opened.
    ReadOnly = 2,

    /// The device is seen be the producer but is not available for access because it is not reachable.
    NoAccess = 3,

    /// The device is already owned/opened by another entity.
    Busy = 4,

    /// The device is already owned/opened by this GenTL Producer with RW access.
    OpenReadWrite = 5,

    /// The device is already owned/opened by this GenTL Producer with RO access.
    OpenReadOnly = 6,
}

impl DeviceAccessStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::ReadWrite => "ReadWrite",
            Self::ReadOnly => "ReadOnly",
            Self::NoAccess => "NoAccess",
            Self::Busy => "Busy",
            Self::OpenReadWrite => "OpenReadWrite",
            Self::OpenReadOnly => "OpenReadOnly",
        }
    }
}

impl TryFrom<i32> for DeviceAccessStatus {
    type Error = GenTlError;
    fn try_from(value: i32) -> GenTlResult<Self> {
        match value {
            _ if value == (Self::Unknown as i32) => Ok(Self::Unknown),
            _ if value == (Self::ReadWrite as i32) => Ok(Self::ReadWrite),
            _ if value == (Self::ReadOnly as i32) => Ok(Self::ReadOnly),
            _ if value == (Self::NoAccess as i32) => Ok(Self::NoAccess),
            _ if value == (Self::Busy as i32) => Ok(Self::Busy),
            _ if value == (Self::OpenReadWrite as i32) => Ok(Self::OpenReadWrite),
            _ if value == (Self::OpenReadOnly as i32) => Ok(Self::OpenReadOnly),
            _ => Err(GenTlError::InvalidValue(
                "Invalid value for DeviceAccessStatus".into(),
            )),
        }
    }
}

/// This enume defines different modes how a device is to be opened with the IFOpenDevice function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeviceAccessFlag {
    /// Opens the device read only. All Port functions can only read from the device.
    ReadOnly,
    /// Opens the device in a way that other hosts/processes can have read only access to
    /// the device. Device access level is read/write for this process.
    Control,
    /// Open the device in a way that only this host/process can have access to the device.
    /// Device access level is read/write for this process.
    Exclusive,
}

impl DeviceAccessStatus {
    pub(crate) fn is_opened(self) -> bool {
        use DeviceAccessStatus::*;

        matches!(self, OpenReadOnly | OpenReadWrite)
    }
}

pub(crate) trait Device: Port {
    /// Open the device and the remote device.
    fn open(&mut self, access_flag: DeviceAccessFlag) -> GenTlResult<()>;

    /// close the device and the remote device.
    fn close(&mut self) -> GenTlResult<()>;

    /// ID of the device module.
    fn device_id(&self) -> &str;

    /// Port of the remote device.
    fn remote_device(&self) -> GenTlResult<&Mutex<dyn Port>>;

    /// Vendor name of the remote device.
    fn vendor_name(&self) -> GenTlResult<String>;

    /// Model name of the remote device.
    fn model_name(&self) -> GenTlResult<String>;

    /// Display name of the remote device.
    /// If this is not defined in the device this should be “VENDOR MODEL (ID)”
    fn display_name(&self) -> GenTlResult<String>;

    /// Transport layer type of the device.
    fn tl_type(&self) -> TlType;

    /// Access status of the device.
    fn device_access_status(&self) -> DeviceAccessStatus;

    /// User defined name of the device.
    /// If the information is not available, return [`GenTlError::NotAvailable`].
    fn user_defined_name(&self) -> GenTlResult<String>;

    /// Serial number of the remote device.
    fn serial_number(&self) -> GenTlResult<String>;

    /// evice version in string format.
    fn device_version(&self) -> GenTlResult<String>;

    /// Tick frequency of the device’s timestamp counter in ticks per second
    fn timespamp_frequency(&self) -> GenTlResult<u64>;
}
