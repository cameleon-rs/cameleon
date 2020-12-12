use super::*;

pub(super) type PORT_HANDLE = *mut libc::c_void;

newtype_enum! {
    pub enum PORT_INFO_CMD {
        PORT_INFO_ID = 0,
        PORT_INFO_VENDOR = 1,
        PORT_INFO_MODEL = 2,
        PORT_INFO_TLTYPE = 3,
        PORT_INFO_MODULE = 4,
        PORT_INFO_LITTLE_ENDIAN = 5,
        PORT_INFO_BIG_ENDIAN = 6,
        PORT_INFO_ACCESS_READ = 7,
        PORT_INFO_ACCESS_WRITE = 8,
        PORT_INFO_ACCESS_NA = 9,
        PORT_INFO_ACCESS_NI = 10,
        PORT_INFO_VERSION = 11,
        PORT_INFO_PORTNAME = 12,
        PORT_INFO_CUSTOM_ID = 1000,
    }
}

newtype_enum! {
    pub enum URL_INFO_CMD {
        URL_INFO_URL = 0,
        URL_INFO_SCHEMA_VER_MAJOR = 1,
        URL_INFO_SCHEMA_VER_MINOR = 2,
        URL_INFO_FILE_VER_MAJOR = 3,
        URL_INFO_FILE_VER_MINOR = 4,
        URL_INFO_FILE_VER_SUBMINOR = 5,
        URL_INFO_FILE_SHA1_HASH = 6,
        URL_INFO_FILE_REGISTER_ADDRESS = 7,
        URL_INFO_FILE_SIZE = 8,
        URL_INFO_SCHEME = 9,
        URL_INFO_FILENAME = 10,
        URL_INFO_CUSTOM_ID = 1000,
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PORT_REGISTER_STACK_ENTRY {
    Address: u64,
    pBuffer: *mut libc::c_void,
    Size: libc::size_t,
}

gentl_api! {
    pub fn GCGetPortInfo(
        hPort: PORT_HANDLE,
        iInfoCmd: PORT_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn GCGetPortURL(
        hPort: PORT_HANDLE,
        sURL: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn GCGetNumPortURLs(hPort: PORT_HANDLE, piNumURLs: *mut u32) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn GCGetPortURLInfo(
        hPort: PORT_HANDLE,
        iURLIndex: u32,
        iInfoCmd: URL_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn GCReadPort(
        hPort: PORT_HANDLE,
        iAddress: u64,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn GCWritePort(
        hPort: PORT_HANDLE,
        iAddress: u64,
        pBuffer: libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}

gentl_api! {
    pub fn GCReadPortStacked(
        hPort: PORT_HANDLE,
        pEntries: *mut PORT_REGISTER_STACK_ENTRY,
        piNumEntries: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
  }
}

gentl_api! {
    pub fn GCWritePortStacked(
        hPort: PORT_HANDLE,
        pEntries: *mut PORT_REGISTER_STACK_ENTRY,
        piNumEntries: *mut libc::size_t,
    ) -> GenTlResult<()> {
        todo!()
    }
}
