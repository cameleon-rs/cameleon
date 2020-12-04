use std::{convert::TryInto, sync::Mutex};

use super::*;

pub(super) type DEV_HANDLE = *mut libc::c_void;
pub(super) type PORT_HANDLE = *mut libc::c_void;
pub(super) type DS_HANDLE = *mut libc::c_void;

#[derive(Clone, Copy)]
pub(super) struct DeviceModuleRef<'a> {
    inner: &'a Mutex<dyn imp::device::Device>,
    parent_if: super::interface::IF_HANDLE,
}

impl<'a> DeviceModuleRef<'a> {
    pub(super) fn new(
        inner: &'a Mutex<dyn imp::device::Device>,
        parent_if: interface::IF_HANDLE,
    ) -> Self {
        Self { inner, parent_if }
    }
}

impl<'a> std::ops::Deref for DeviceModuleRef<'a> {
    type Target = Mutex<dyn imp::device::Device>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(super) fn dev_get_info(
    iface: DeviceModuleRef,
    iInfoCmd: DEVICE_INFO_CMD,
    piType: *mut INFO_DATATYPE,
    pBuffer: *mut libc::c_void,
    piSize: *mut libc::size_t,
) -> GenTlResult<()> {
    todo!()
}

newtype_enum! {
    pub enum DEVICE_INFO_CMD {
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_ID = 0,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_VENDOR = 1,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_MODEL = 2,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_TLTYPE = 3,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_DISPLAYNAME = 4,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_ACCESS_STATUS = 5,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_USER_DEFINED_NAME = 6,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_SERIAL_NUMBER = 7,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_VERSION = 8,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_TIMESTAMP_FREQUENCY = 9,
        DEVICE_INFO_CMD_LIST_DEVICE_INFO_CUSTOM_ID = 1000,
    }
}

newtype_enum! {
    pub enum DEVICE_ACCESS_FLAGS {
        DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_UNKNOWN = 0,
        DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_NONE = 1,
        DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_READONLY = 2,
        DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_CONTROL = 3,
        DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_EXCLUSIVE = 4,
        DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_CUSTOM_ID = 1000,
    }
}

impl TryInto<imp::device::DeviceAccessFlag> for DEVICE_ACCESS_FLAGS {
    type Error = GenTlError;

    fn try_into(self) -> GenTlResult<imp::device::DeviceAccessFlag> {
        use imp::device::DeviceAccessFlag::*;
        match self {
            DEVICE_ACCESS_FLAGS::DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_READONLY => Ok(ReadOnly),
            DEVICE_ACCESS_FLAGS::DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_CONTROL => Ok(Control),
            DEVICE_ACCESS_FLAGS::DEVICE_ACCESS_FLAGS_LIST_DEVICE_ACCESS_EXCLUSIVE => Ok(Exclusive),
            _ => Err(GenTlError::InvalidParameter),
        }
    }
}

gentl_api! {
    pub fn DevClose(hDevice: DEV_HANDLE) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn DevGetInfo(
        hDevice: DEV_HANDLE,
        iInfoCmd: DEVICE_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn DevGetDataStreamID(
        hDevice: DEV_HANDLE,
        iIndex: u32,
        sDataStreamID: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn DevGetNumDataStreams(hDevice: DEV_HANDLE, piNumDataStreams: *mut u32) -> GenTlResult<()>
    {
        todo!()
    }
}

gentl_api! {
    pub fn DevGetPort(hDevice: DEV_HANDLE, phRemoteDevice: *mut PORT_HANDLE) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn DevOpenDataStream(
        hDevice: DEV_HANDLE,
        sDataStreamID: *const ::std::os::raw::c_char,
        phDataStream: *mut DS_HANDLE,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn DevGetParentIF(hDevice: DEV_HANDLE, phIface: *mut interface::IF_HANDLE) -> GenTlResult<()> {
        todo!()
    }
}
