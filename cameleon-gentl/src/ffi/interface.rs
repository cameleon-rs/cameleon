use std::sync::Mutex;

use crate::imp;

use super::*;

pub(super) type IF_HANDLE = *mut libc::c_void;

pub(super) type InterfaceModule = Mutex<dyn imp::interface::Interface>;

newtype_enum! {
    pub enum INTERFACE_INFO_CMD {
        /// Unique ID of the interface.
        INTERFACE_INFO_ID = 0,

        /// User readable name of the interface.
        INTERFACE_INFO_DISPLAY_NAME = 1,

        /// Transport layer technology that is supported.
        INTERFACE_INFO_TLTYPE = 2,
    }
}

gentl_api! {
    pub fn IFClose(hIface: IF_HANDLE) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFGetInfo(
        hIface: IF_HANDLE,
        iInfoCmd: INTERFACE_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut ::std::os::raw::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFGetDeviceID(
        hIface: IF_HANDLE,
        iIndex: u32,
        sIDeviceID: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFGetDeviceInfo(
        hIface: IF_HANDLE,
        sDeviceID: *const ::std::os::raw::c_char,
        iInfoCmd: device::DEVICE_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFGetNumDevices(hIface: IF_HANDLE, piNumDevices: *mut u32) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFOpenDevice(
        hIface: IF_HANDLE,
        sDeviceID: *const libc::c_char,
        iOpenFlag: device::DEVICE_ACCESS_FLAGS,
        phDevice: *mut device::DEV_HANDLE,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFUpdateDeviceList(
        hIface: IF_HANDLE,
        pbChanged: *mut bool8_t,
        iTimeout: u64,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn IFGetParentTL(hIface: IF_HANDLE, phSystem: *mut system::TL_HANDLE) -> GenTlResult<()> {
        todo!()
    }
}
