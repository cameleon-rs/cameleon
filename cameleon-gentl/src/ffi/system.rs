use std::mem::ManuallyDrop;

use crate::{GenTlError, GenTlResult};

use super::*;

type TL_HANDLE = *mut libc::c_void;

gentl_api!(
    pub fn TLClose(hSystem: TL_HANDLE) -> GenTlResult<()> {
        let handle = unsafe { HandleType::from_raw_manually_drop(hSystem)? };

        let _ = handle.system()?;
        // Drop handle.
        ManuallyDrop::into_inner(handle);
        Ok(())
    }
);

//gentl_api!(
//    pub fn TLGetInfo(
//        hSystem: TL_HANDLE,
//        iInfoCmd: TL_INFO_CMD,
//        piType: *mut INFO_DATATYPE,
//        pBuffer: *mut libc::c_void,
//        piSize: *mut libc::size_t,
//    ) -> GenTlResult<()> {
//        let handle = unsafe { HandleType::from_raw_manually_drop(hSystem)? };
//        let system_handle = handle.system()?;
//
//        let (info, info_data_type) = match iInfoCmd {
//            TL_INFO_CMD::TL_INFO_ID => {
//                todo!()
//            }
//            _ => return Err(GenTlError::InvalidParameter),
//        };
//
//        Ok(())
//    }
//);

newtype_enum! {
    pub enum TL_INFO_CMD {
        TL_INFO_ID = 0,
        TL_INFO_VENDOR = 1,
        TL_INFO_MODEL = 2,
        TL_INFO_VERSION = 3,
        TL_INFO_TLTYPE = 4,
        TL_INFO_NAME = 5,
        TL_INFO_PATHNAME = 6,
        TL_INFO_DISPLAYNAME = 7,
        TL_INFO_CHAR_ENCODING = 8,
        TL_INFO_GENTL_VER_MAJOR = 9,
        TL_INFO_GENTL_VER_MINOR = 10,
    }
}
