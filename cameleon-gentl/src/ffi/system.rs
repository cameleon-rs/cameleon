use std::{ffi::CStr, sync::Mutex};

use crate::{imp, GenTlError, GenTlResult};

use super::*;

type TL_HANDLE = *mut libc::c_void;

pub(super) struct SystemModule(Mutex<Option<imp::system::SystemModule>>);

impl SystemModule {
    fn new() -> Self {
        SystemModule(Mutex::new(None))
    }

    fn open(&self) -> GenTlResult<()> {
        let mut system_module = self.0.lock().unwrap();
        if system_module.is_some() {
            return Err(GenTlError::ResourceInUse);
        }

        *system_module = Some(imp::system::SystemModule::new()?);
        Ok(())
    }

    fn close(&self) -> GenTlResult<()> {
        *self.0.lock().unwrap() = None;
        Ok(())
    }

    fn is_opened(&self) -> bool {
        (&mut *self.0.lock().unwrap()).is_some()
    }

    fn with<F, U>(&self, f: F) -> GenTlResult<U>
    where
        F: FnOnce(&mut imp::system::SystemModule) -> GenTlResult<U>,
    {
        let mut module = self.0.lock().unwrap();
        match &mut *module {
            Some(ref mut system) => f(system),
            None => Err(GenTlError::NotInitialized),
        }
    }
}

lazy_static::lazy_static! {
    static ref SYSTEM_MODULE: Box<SystemModule> = Box::new(SystemModule::new());
}

gentl_api! {
    pub fn TLOpen(phSystem: *mut TL_HANDLE) -> GenTlResult<()> {
        SYSTEM_MODULE.open()?;

        let handle = Box::new(ModuleHandle::System(SYSTEM_MODULE.as_ref()));
        unsafe {
            *phSystem = handle.into_raw();
        }
        Ok(())
    }
}

gentl_api!(
    pub fn TLClose(hSystem: TL_HANDLE) -> GenTlResult<()> {
        let mut handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let _ = handle.system()?;

        // Close module.
        SYSTEM_MODULE.close()?;
        // Drop handle itself.
        unsafe {
            std::mem::ManuallyDrop::drop(&mut handle);
        }
        Ok(())
    }
);

gentl_api!(
    pub fn TLGetInfo(
        hSystem: TL_HANDLE,
        iInfoCmd: TL_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let system_handle = handle.system()?;
        let (size, info_data_type) = system_handle.with(|handle| {
            let system_info = handle.system_info();

            let res = match iInfoCmd {
                TL_INFO_CMD::TL_INFO_ID => {
                    copy_info(system_info.id.as_str(), pBuffer as *mut libc::c_char)
                }
                TL_INFO_CMD::TL_INFO_VENDOR => {
                    copy_info(system_info.vendor.as_str(), pBuffer as *mut libc::c_char)
                }
                TL_INFO_CMD::TL_INFO_MODEL => {
                    copy_info(system_info.model.as_str(), pBuffer as *mut libc::c_char)
                }
                TL_INFO_CMD::TL_INFO_VERSION => {
                    copy_info(system_info.version.as_str(), pBuffer as *mut libc::c_char)
                }
                TL_INFO_CMD::TL_INFO_TLTYPE => {
                    copy_info(system_info.tl_type.as_str(), pBuffer as *mut libc::c_char)
                }
                TL_INFO_CMD::TL_INFO_NAME => copy_info(
                    &*system_info.full_path.file_name().unwrap().to_string_lossy(),
                    pBuffer as *mut libc::c_char,
                ),
                TL_INFO_CMD::TL_INFO_PATHNAME => copy_info(
                    &*system_info.full_path.to_string_lossy(),
                    pBuffer as *mut libc::c_char,
                ),
                TL_INFO_CMD::TL_INFO_DISPLAYNAME => copy_info(
                    system_info.display_name.as_str(),
                    pBuffer as *mut libc::c_char,
                ),
                TL_INFO_CMD::TL_INFO_CHAR_ENCODING => {
                    copy_info(system_info.encoding.as_raw(), pBuffer as *mut i32)
                }
                TL_INFO_CMD::TL_INFO_GENTL_VER_MAJOR => {
                    copy_info(system_info.gentl_version_major, pBuffer as *mut u32)
                }
                TL_INFO_CMD::TL_INFO_GENTL_VER_MINOR => {
                    copy_info(system_info.gentl_version_minor, pBuffer as *mut u32)
                }
                _ => return Err(GenTlError::InvalidParameter),
            }?;

            Ok(res)
        })?;

        unsafe {
            *piSize = size;
            *piType = info_data_type;
        }

        Ok(())
    }
);

gentl_api! {
    pub fn TLGetInterfaceID(
        hSystem: TL_HANDLE,
        iIndex: u32,
        sIfaceID: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let system_handle = handle.system()?;

        system_handle.with(|handle| {
            let iface = handle.interfaces().nth(iIndex as usize).ok_or(GenTlError::InvalidIndex)?;
            let iface_guard = iface.lock().unwrap();
            let id = iface_guard.interface_id();

            unsafe {
                *piSize = id.copy_to(sIfaceID)?;
            }
            Ok(())
        })
    }
}

gentl_api! {
    pub fn TLGetInterfaceInfo(
        hSystem: TL_HANDLE,
        sIfaceID: *const libc::c_char,
        iInfoCmd: interface::INTERFACE_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let system_handle = handle.system()?;
        let id = unsafe {CStr::from_ptr(sIfaceID)}.to_string_lossy();

        system_handle.with(|handle| {
            use  interface::INTERFACE_INFO_CMD;
            let iface = handle.interface_of(&id).ok_or_else(||GenTlError::InvalidId(id.into()))?;
            let (size, info_data_type) = match iInfoCmd {
                INTERFACE_INFO_CMD::INTERFACE_INFO_ID =>
                    copy_info(iface.lock().unwrap().interface_id(), pBuffer as *mut libc::c_char),
                INTERFACE_INFO_CMD::INTERFACE_INFO_DISPLAY_NAME =>
                    copy_info(iface.lock().unwrap().display_name(), pBuffer as *mut libc::c_char),
                INTERFACE_INFO_CMD::INTERFACE_INFO_TLTYPE =>
                    copy_info(iface.lock().unwrap().tl_type(), pBuffer as *mut libc::c_char),
                _ => Err(GenTlError::InvalidParameter),
            }?;

            unsafe {
                *piSize = size;
                *piType = info_data_type;
            }

            Ok(())
        })
    }

}

gentl_api! {
    pub fn TLGetNumInterfaces(hSystem: TL_HANDLE, piNumIfaces: *mut u32) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let system_handle = handle.system()?;

        system_handle.with(|handle| {
            let ifaces = handle.interfaces();
            unsafe {
                *piNumIfaces = ifaces.count() as u32;
            }
            Ok(())
        })
    }
}

gentl_api! {
    pub fn TLOpenInterface(
        hSystem: TL_HANDLE,
        sIfaceID: *const libc::c_char,
        phIface: *mut super::interface::IF_HANDLE,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let system_handle = handle.system()?;
        let id = unsafe {CStr::from_ptr(sIfaceID)}.to_string_lossy();

        system_handle.with(|handle| {
            let iface = handle.interface_of(&id).ok_or_else(|| GenTlError::InvalidId(id.into()))?;
            iface.lock().unwrap().open()?;
            let module_handle = Box::new(ModuleHandle::Interface(iface));

            unsafe {
                *phIface = module_handle.into_raw();
            }

            Ok(())
        })
    }
}

gentl_api! {
    pub fn TLUpdateInterfaceList(
        hSystem: TL_HANDLE,
        pbChanged: *mut bool8_t,
        _iTimeout: u64,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hSystem)? };
        let system_handle = handle.system()?;
        if system_handle.is_opened() {
            unsafe {
                *pbChanged = bool8_t::false_();
            }

            Ok(())
        } else {
            Err(GenTlError::NotInitialized)
        }
    }
}

newtype_enum! {
    pub enum TL_INFO_CMD {
        /// Unique ID identifying a GenTL Producer"
        TL_INFO_ID = 0,

        /// GenTL Producer vendor name.
        TL_INFO_VENDOR = 1,

        /// GenTL Producer model name.
        TL_INFO_MODEL = 2,

        /// GenTL Producer version.
        TL_INFO_VERSION = 3,

        /// Transport layer technology that is supported.
        TL_INFO_TLTYPE = 4,

        /// File name of the system module.
        TL_INFO_NAME = 5,

        /// Full path to the system module.
        TL_INFO_PATHNAME = 6,

        /// User readable name of the GenTL Producer.
        TL_INFO_DISPLAYNAME = 7,

        /// The char encoding of the GenTL Producer.
        TL_INFO_CHAR_ENCODING = 8,

        /// Major version number of GenTL Standard Version this Producer complies with.
        TL_INFO_GENTL_VER_MAJOR = 9,

        /// Minor version number of GenTL Standard Version this Producer complies with.
        TL_INFO_GENTL_VER_MINOR = 10,
    }
}
