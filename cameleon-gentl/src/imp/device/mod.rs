use crate::GenTlResult;

pub(crate) mod u3v;

mod u3v_memory;

/// The current accessibility of the device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeviceAccessStatus {
    /// The current availability of the device is unknown.
    Unknown,

    /// The device is available to be opened for Read/Write access but it is currently not opened.
    ReadWrite,

    /// The device is available to be opened for Read access but is currently not opened.
    ReadOnly,

    /// The device is seen be the producer but is not available for access because it is not reachable.
    NoAccess,

    /// The device is already owned/opened by another entity.
    Busy,

    /// The device is already owned/opened by this GenTL Producer with RW access.
    OpenReadWrite,

    /// The device is already owned/opened by this GenTL Producer with RO access.
    OpenReadOnly,
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

pub(crate) trait Device {
    fn open(&mut self, access_flag: DeviceAccessFlag) -> GenTlResult<()>;

    fn close(&mut self) -> GenTlResult<()>;

    fn device_id(&self) -> &str;
}
