use std::{convert::TryInto, sync::Mutex};

use super::*;

pub(super) type DEV_HANDLE = *mut libc::c_void;

pub(super) type DeviceModule = Mutex<dyn imp::device::Device>;

pub(super) fn dev_get_info(
    iface: &DeviceModule,
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
