use std::sync::Mutex;

use crate::imp;

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
