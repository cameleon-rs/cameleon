/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{convert::TryInto, ffi::CStr, sync::Mutex, time::Duration};

use crate::imp;

use super::{
    bool8_t, copy_info, device, device::DeviceModuleRef, system, CopyTo, GenTlError, GenTlResult,
    ModuleHandle, GC_ERROR, INFO_DATATYPE,
};

pub(super) type IF_HANDLE = *mut libc::c_void;

#[derive(Clone, Copy)]
pub(super) struct InterfaceModuleRef<'a> {
    inner: &'a Mutex<dyn imp::interface::Interface>,
    parent_tl: super::system::TL_HANDLE,
}

impl<'a> InterfaceModuleRef<'a> {
    pub(super) fn new(
        inner: &'a Mutex<dyn imp::interface::Interface>,
        parent_tl: super::system::TL_HANDLE,
    ) -> Self {
        Self { inner, parent_tl }
    }
}

impl<'a> std::ops::Deref for InterfaceModuleRef<'a> {
    type Target = Mutex<dyn imp::interface::Interface>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(super) fn if_get_info(
    iface: InterfaceModuleRef,
    iInfoCmd: INTERFACE_INFO_CMD,
    piType: *mut INFO_DATATYPE,
    pBuffer: *mut libc::c_void,
    piSize: *mut libc::size_t,
) -> GenTlResult<()> {
    let iface_guard = iface.lock().unwrap();
    let info_data_type = match iInfoCmd {
        INTERFACE_INFO_CMD::INTERFACE_INFO_ID => {
            copy_info(iface_guard.interface_id(), pBuffer, piSize)
        }

        INTERFACE_INFO_CMD::INTERFACE_INFO_DISPLAY_NAME => {
            copy_info(iface_guard.display_name(), pBuffer, piSize)
        }

        INTERFACE_INFO_CMD::INTERFACE_INFO_TLTYPE => {
            copy_info(iface_guard.tl_type(), pBuffer, piSize)
        }

        _ => Err(GenTlError::InvalidParameter),
    }?;

    unsafe {
        *piType = info_data_type;
    }

    Ok(())
}

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
        let mut handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface_handle = handle.interface()?;

        // Close the interface module.
        iface_handle.lock().unwrap().close()?;
        // Drop its handle.
        unsafe {
            std::mem::ManuallyDrop::drop(&mut handle);
        }

        Ok(())
    }
}

gentl_api! {
    pub fn IFGetInfo(
        hIface: IF_HANDLE,
        iInfoCmd: INTERFACE_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        if_get_info(iface, iInfoCmd, piType, pBuffer, piSize)
    }
}

gentl_api! {
    pub fn IFGetDeviceID(
        hIface: IF_HANDLE,
        iIndex: u32,
        sIDeviceID: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        let iface_guard = iface.lock().unwrap();
        let devices = iface_guard.devices();
        let device = devices
            .get(iIndex as usize)
            .ok_or(GenTlError::InvalidIndex)?;

        device.lock().unwrap().device_id().copy_to(sIDeviceID, piSize)?;

        Ok(())
    }
}

gentl_api! {
    pub fn IFGetDeviceInfo(
        hIface: IF_HANDLE,
        sDeviceID: *const libc::c_char,
        iInfoCmd: device::DEVICE_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        let iface_guard = iface.lock().unwrap();
        let id = unsafe { CStr::from_ptr(sDeviceID) }.to_string_lossy();
        let device = iface_guard.device_by_id(&id)?;

        device::dev_get_info(device, iInfoCmd, piType, pBuffer, piSize)
    }
}

gentl_api! {
    pub fn IFGetNumDevices(hIface: IF_HANDLE, piNumDevices: *mut u32) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        let device_num = iface.lock().unwrap().devices().len();
        unsafe {
            *piNumDevices = device_num as u32;
        }

        Ok(())
    }
}

gentl_api! {
    pub fn IFOpenDevice(
        hIface: IF_HANDLE,
        sDeviceID: *const libc::c_char,
        iOpenFlag: device::DEVICE_ACCESS_FLAGS,
        phDevice: *mut device::DEV_HANDLE,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        let iface_guard = iface.lock().unwrap();
        let id = unsafe { CStr::from_ptr(sDeviceID) }.to_string_lossy();
        let device = iface_guard.device_by_id(&id)?;

        device.lock().unwrap().open(iOpenFlag.try_into()?)?;
        let device = DeviceModuleRef::new(device, hIface)?;
        let device_handle = Box::new(ModuleHandle::Device(device));
        unsafe {
            *phDevice = device_handle.into_raw();
        }

        Ok(())
    }
}

gentl_api! {
    pub fn IFUpdateDeviceList(
        hIface: IF_HANDLE,
        pbChanged: *mut bool8_t,
        iTimeout: u64,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        let mut iface_guard = iface.lock().unwrap();
        let timeout = Duration::from_millis(iTimeout);

        let is_changed = iface_guard.update_device_list(timeout)?.into();
        unsafe {
            *pbChanged = is_changed;
        }

        Ok(())
    }
}

gentl_api! {
    pub fn IFGetParentTL(hIface: IF_HANDLE, phSystem: *mut system::TL_HANDLE) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hIface)? };
        let iface = handle.interface()?;

        unsafe {
            *phSystem = iface.parent_tl;
        }

        Ok(())
    }
}
