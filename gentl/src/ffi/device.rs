use std::{convert::TryInto, ops::Deref, sync::Mutex};

use super::{
    copy_info, imp, interface, GenTlError, GenTlResult, ModuleHandle, GC_ERROR, INFO_DATATYPE,
};

pub(super) type DEV_HANDLE = *mut libc::c_void;
pub(super) type PORT_HANDLE = *mut libc::c_void;
pub(super) type DS_HANDLE = *mut libc::c_void;

#[derive(Clone, Copy)]
pub(super) struct DeviceModuleRef<'a> {
    inner: &'a Mutex<dyn imp::device::Device>,
    parent_if: super::interface::IF_HANDLE,
    remote_handle: PORT_HANDLE,
}

impl<'a> DeviceModuleRef<'a> {
    pub(super) fn new(
        inner: &'a Mutex<dyn imp::device::Device>,
        parent_if: interface::IF_HANDLE,
    ) -> GenTlResult<Self> {
        let dev_guard = inner.lock().unwrap();

        let remote_device = RemoteDeviceRef {
            inner: dev_guard.remote_device()?,
        };

        let remote_handle =
            unsafe { Box::new(ModuleHandle::RemoteDevice(remote_device)).into_raw() };

        Ok(Self {
            inner,
            parent_if,
            remote_handle,
        })
    }
}

impl<'a> Deref for DeviceModuleRef<'a> {
    type Target = Mutex<dyn imp::device::Device>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

#[derive(Clone, Copy)]
pub(super) struct RemoteDeviceRef<'a> {
    inner: &'a Mutex<dyn imp::port::Port>,
}

impl<'a> Deref for RemoteDeviceRef<'a> {
    type Target = Mutex<dyn imp::port::Port>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

pub(super) fn dev_get_info(
    dev: impl Deref<Target = Mutex<dyn imp::device::Device>>,
    iInfoCmd: DEVICE_INFO_CMD,
    piType: *mut INFO_DATATYPE,
    pBuffer: *mut libc::c_void,
    piSize: *mut libc::size_t,
) -> GenTlResult<()> {
    let dev_guard = dev.lock().unwrap();
    let info_data_type = match iInfoCmd {
        DEVICE_INFO_CMD::DEVICE_INFO_ID => copy_info(dev_guard.device_id(), pBuffer, piSize),

        DEVICE_INFO_CMD::DEVICE_INFO_VENDOR => {
            copy_info(dev_guard.vendor_name()?.as_str(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_MODEL => {
            copy_info(dev_guard.model_name()?.as_str(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_TLTYPE => copy_info(dev_guard.tl_type(), pBuffer, piSize),

        DEVICE_INFO_CMD::DEVICE_INFO_DISPLAYNAME => {
            copy_info(dev_guard.display_name()?.as_str(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_ACCESS_STATUS => {
            copy_info(dev_guard.device_access_status(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_USER_DEFINED_NAME => {
            copy_info(dev_guard.user_defined_name()?.as_str(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_SERIAL_NUMBER => {
            copy_info(dev_guard.serial_number()?.as_str(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_VERSION => {
            copy_info(dev_guard.device_version()?.as_str(), pBuffer, piSize)
        }

        DEVICE_INFO_CMD::DEVICE_INFO_TIMESTAMP_FREQUENCY => {
            copy_info(dev_guard.timespamp_frequency()?, pBuffer, piSize)
        }

        _ => Err(GenTlError::InvalidParameter),
    }?;

    unsafe {
        *piType = info_data_type;
    }

    Ok(())
}

newtype_enum! {
    pub enum DEVICE_INFO_CMD {
        DEVICE_INFO_ID = 0,
        DEVICE_INFO_VENDOR = 1,
        DEVICE_INFO_MODEL = 2,
        DEVICE_INFO_TLTYPE = 3,
        DEVICE_INFO_DISPLAYNAME = 4,
        DEVICE_INFO_ACCESS_STATUS = 5,
        DEVICE_INFO_USER_DEFINED_NAME = 6,
        DEVICE_INFO_SERIAL_NUMBER = 7,
        DEVICE_INFO_VERSION = 8,
        DEVICE_INFO_TIMESTAMP_FREQUENCY = 9,
        DEVICE_INFO_CUSTOM_ID = 1000,
    }
}

newtype_enum! {
    pub enum DEVICE_ACCESS_FLAGS {
        DEVICE_ACCESS_UNKNOWN = 0,
        DEVICE_ACCESS_NONE = 1,
        DEVICE_ACCESS_READONLY = 2,
        DEVICE_ACCESS_CONTROL = 3,
        DEVICE_ACCESS_EXCLUSIVE = 4,
        DEVICE_ACCESS_CUSTOM_ID = 1000,
    }
}

impl TryInto<imp::device::DeviceAccessFlag> for DEVICE_ACCESS_FLAGS {
    type Error = GenTlError;

    fn try_into(self) -> GenTlResult<imp::device::DeviceAccessFlag> {
        use imp::device::DeviceAccessFlag::{Control, Exclusive, ReadOnly};
        match self {
            DEVICE_ACCESS_FLAGS::DEVICE_ACCESS_READONLY => Ok(ReadOnly),
            DEVICE_ACCESS_FLAGS::DEVICE_ACCESS_CONTROL => Ok(Control),
            DEVICE_ACCESS_FLAGS::DEVICE_ACCESS_EXCLUSIVE => Ok(Exclusive),
            _ => Err(GenTlError::InvalidParameter),
        }
    }
}

gentl_api! {
    pub fn DevClose(hDevice: DEV_HANDLE) -> GenTlResult<()> {
        let mut handle = unsafe { ModuleHandle::from_raw_manually_drop(hDevice)? };
        let dev_handle = handle.device()?;

        // Close the device module.
        dev_handle.lock().unwrap().close()?;

        // Drop remote device handle.
        // This seems weired but there is no function to close remote device in GenTL API.
        unsafe {
                let mut remote_handle = ModuleHandle::from_raw_manually_drop(dev_handle.remote_handle)?;
                std::mem::ManuallyDrop::drop(&mut remote_handle);
        }

        // Drop the device handle.
        unsafe {
            std::mem::ManuallyDrop::drop(&mut handle)
        }

        Ok(())
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
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hDevice)? };
        let dev_handle = handle.device()?;

        dev_get_info(dev_handle, iInfoCmd, piType, pBuffer, piSize)
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
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hDevice)? };
        let dev_handle = handle.device()?;

        unsafe {
            *phRemoteDevice = dev_handle.remote_handle;
        }

        Ok(())
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
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hDevice)? };
        let dev_handle = handle.device()?;

        unsafe {
            *phIface = dev_handle.parent_if;
        }

        Ok(())
    }
}
