use std::slice;

use imp::port::Port as _;

use super::{
    bool8_t, copy_info, imp, GenTlError, GenTlResult, ModuleHandle, GC_ERROR, INFO_DATATYPE,
};

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

newtype_enum! {
    pub enum URL_SCHEME_IDS {
        URL_SCHEME_LOCAL = 0,
        URL_SCHEME_HTTP = 1,
        URL_SCHEME_FILE = 2,
    }
}

macro_rules! with_port {
    ($handle:ident, |$port:ident| $body: tt) => {
        match &*$handle.as_ref() {
            ModuleHandle::System(handle) => {
                #[allow(unused_mut)]
                let mut $port = handle.lock().unwrap();
                $body
            }

            ModuleHandle::Interface(handle) => {
                let mut $port = handle.lock().unwrap();
                $body
            }

            ModuleHandle::Device(handle) => {
                let mut $port = handle.lock().unwrap();
                $body
            }

            ModuleHandle::RemoteDevice(handle) => {
                let mut $port = handle.lock().unwrap();
                $body
            }
        }
    };
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PORT_REGISTER_STACK_ENTRY {
    Address: u64,
    pBuffer: *mut libc::c_void,
    Size: libc::size_t,
}

fn file_location_to_url(xml_info: &imp::port::XmlInfo, port_info: &imp::port::PortInfo) -> String {
    use imp::port::XmlLocation;
    match &xml_info.location {
        XmlLocation::RegisterMap { address, size } => {
            // The specificaton requires us to stringfy xml location as below when the
            // xml is On the register,
            //
            // local:{filename}.{extension};{address};{length}[?SchemaVersion={major}.(minor).{subminor}]
            //
            // filename: {vendor}_{model}_{file_version}.
            // extension: "zip" if compressed else "xml".
            // address: Start address of the xml, must be expressed in hexademical
            // without prefix.
            // length: Byte length  of the xml, must be expressed in hexademical
            // without prefix.
            let extension = match xml_info.compressed {
                cameleon::device::CompressionType::Uncompressed => "xml",
                cameleon::device::CompressionType::Zip => "zip",
            };
            let schema_version = &xml_info.schema_version;

            format!("local:{vendor}_{model}_{file_version}.{extension};{address:X};{size:X}?SchemaVersion={schema_major}.{schema_minor}.{schema_subminor}",
                            vendor = port_info.vendor,
                            model = port_info.model,
                            file_version = xml_info.file_version,
                            extension = extension,
                            address = address,
                            size = size,
                            schema_major=schema_version.major,
                            schema_minor = schema_version.minor,
                            schema_subminor = schema_version.patch,
                        )
        }
        XmlLocation::LocalFile(path) => {
            // file:{filepath}[?SchemaVersion={major}.{minor}.{subminor}]
            let schema_version = &xml_info.schema_version;
            format!(
                "file:{filepath}?SchemaVersion={schema_major}.{schema_minor}.{schema_subminor}",
                filepath = path.to_string_lossy(),
                schema_major = schema_version.major,
                schema_minor = schema_version.minor,
                schema_subminor = schema_version.patch,
            )
        }
        XmlLocation::Url(url) => {
            // {url}[?SchemaVersion={major}.{minor}.{subminor}]
            let schema_version = &xml_info.schema_version;
            format!(
                "{url}?SchemaVersion={schema_major}.{schema_minor}.{schema_subminor}",
                url = url.as_str(),
                schema_major = schema_version.major,
                schema_minor = schema_version.minor,
                schema_subminor = schema_version.patch,
            )
        }
    }
}

gentl_api! {
    pub fn GCGetPortInfo(
        hPort: PORT_HANDLE,
        iInfoCmd: PORT_INFO_CMD,
        piType: *mut INFO_DATATYPE,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hPort)? };

        let info_data_type = with_port!(handle, |port| {
            let info = port.port_info()?;
            match iInfoCmd {
                PORT_INFO_CMD::PORT_INFO_ID => copy_info(info.id.as_str(), pBuffer, piSize),

                PORT_INFO_CMD::PORT_INFO_VENDOR => copy_info(info.vendor.as_str(), pBuffer, piSize),

                PORT_INFO_CMD::PORT_INFO_MODEL => copy_info(info.model.as_str(), pBuffer, piSize),

                PORT_INFO_CMD::PORT_INFO_TLTYPE => copy_info(info.tl_type, pBuffer, piSize),

                PORT_INFO_CMD::PORT_INFO_MODULE => copy_info(info.module_type, pBuffer, piSize),

                PORT_INFO_CMD::PORT_INFO_LITTLE_ENDIAN => {
                    let is_le: bool8_t = (info.endianness == imp::port::Endianness::LE).into();
                    copy_info(is_le, pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_BIG_ENDIAN => {
                    let is_le: bool8_t = (info.endianness == imp::port::Endianness::BE).into();
                    copy_info(is_le, pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_ACCESS_READ => {
                    let is_readable: bool8_t = info.access.is_readable().into();
                    copy_info(is_readable, pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_ACCESS_WRITE => {
                    let is_writable: bool8_t = info.access.is_writable().into();
                    copy_info(is_writable, pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_ACCESS_NA => {
                    let is_na: bool8_t = (info.access == imp::port::PortAccess::NA).into();
                    copy_info(is_na, pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_ACCESS_NI => {
                    let is_ni: bool8_t = (info.access == imp::port::PortAccess::NI).into();
                    copy_info(is_ni, pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_VERSION => {
                    let version = format!("{}", info.version);
                    copy_info(version.as_str(), pBuffer, piSize)
                }

                PORT_INFO_CMD::PORT_INFO_PORTNAME => {
                    copy_info(info.port_name.as_str(), pBuffer, piSize)
                }

                _ => Err(GenTlError::InvalidParameter),
            }
        })?;

        unsafe {
            *piType = info_data_type;
        }

        Ok(())
    }
}

gentl_api! {
    pub fn GCGetPortURL(
        hPort: PORT_HANDLE,
        sURL: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hPort)? };
        let url = with_port!(handle, |port| {
            // Use first  info.
            let xml_info = port.xml_infos()?.get(0).ok_or_else(|| GenTlError::Error("no xml information in the device".into()))?;
            file_location_to_url(xml_info, port.port_info()?)
        });

        copy_info(url.as_str(), sURL.cast::<libc::c_void>(), piSize)?;
        Ok(())
    }
}

gentl_api! {
    pub fn GCGetNumPortURLs(hPort: PORT_HANDLE, piNumURLs: *mut u32) -> GenTlResult<()> {
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hPort)? };
        let num_port = with_port!{handle, |port| {
            let xml_infos = port.xml_infos()?;
            xml_infos.len()
        }};

        unsafe {
            *piNumURLs = num_port as u32;
        }

        Ok(())
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
        let handle = unsafe { ModuleHandle::from_raw_manually_drop(hPort)? };
        let info_data_type = with_port!(handle, |port| {
            let info = port
                .xml_infos()?
                .get(iURLIndex as usize)
                .ok_or(GenTlError::InvalidIndex)?;
            match iInfoCmd {
                URL_INFO_CMD::URL_INFO_URL => {
                    let url = file_location_to_url(info, port.port_info()?);
                    copy_info(url.as_str(), pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_SCHEMA_VER_MAJOR => {
                    let schema_version = &info.schema_version;
                    copy_info(schema_version.major as i32, pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_SCHEMA_VER_MINOR => {
                    let schema_version = &info.schema_version;
                    copy_info(schema_version.minor as i32, pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_FILE_VER_MAJOR => {
                    let file_version = &info.file_version;
                    copy_info(file_version.major as i32, pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_FILE_VER_MINOR => {
                    let file_version = &info.file_version;
                    copy_info(file_version.minor as i32, pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_FILE_VER_SUBMINOR => {
                    let file_version = &info.file_version;
                    copy_info(file_version.patch as i32, pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_FILE_SHA1_HASH => match info.sha1_hash {
                    Some(hash) => copy_info(hash.as_ref(), pBuffer, piSize),
                    None => Err(GenTlError::NotAvailable),
                },

                URL_INFO_CMD::URL_INFO_FILE_REGISTER_ADDRESS => {
                    use imp::port::XmlLocation;

                    if let XmlLocation::RegisterMap { address, .. } = &info.location {
                        copy_info(*address, pBuffer, piSize)
                    } else {
                        Err(GenTlError::NotAvailable)
                    }
                }

                URL_INFO_CMD::URL_INFO_FILE_SIZE => {
                    use imp::port::XmlLocation;

                    if let XmlLocation::RegisterMap { size, .. } = &info.location {
                        copy_info(*size as u64, pBuffer, piSize)
                    } else {
                        Err(GenTlError::NotAvailable)
                    }
                }

                URL_INFO_CMD::URL_INFO_SCHEME => {
                    use imp::port::XmlLocation;
                    let scheme_id = match &info.location {
                        XmlLocation::RegisterMap { .. } => URL_SCHEME_IDS::URL_SCHEME_LOCAL,
                        XmlLocation::LocalFile(_) => URL_SCHEME_IDS::URL_SCHEME_FILE,
                        XmlLocation::Url(_) => URL_SCHEME_IDS::URL_SCHEME_HTTP,
                    };

                    copy_info(scheme_id.0, pBuffer, piSize)
                }

                URL_INFO_CMD::URL_INFO_FILENAME => {
                    use imp::port::XmlLocation;
                    let file_name = match &info.location {
                        XmlLocation::LocalFile(path) => {
                            let err_msg = "local file name is invalid";
                            let file_name =
                                path.file_name().ok_or_else(|| GenTlError::Error(err_msg.into()))?;
                            file_name
                                .to_str()
                                .ok_or_else(|| GenTlError::Error(err_msg.into()))?
                        }

                        _ => return Err(GenTlError::NotAvailable),
                    };

                    copy_info(file_name, pBuffer, piSize)
                }

                _ => Err(GenTlError::InvalidParameter),
            }
        })?;

        unsafe {
            *piType = info_data_type;
        }

        Ok(())
    }
}

gentl_api! {
    pub fn GCReadPort(
        hPort: PORT_HANDLE,
        iAddress: u64,
        pBuffer: *mut libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        unsafe {
            let handle = ModuleHandle::from_raw_manually_drop(hPort)?;
            let buffer = std::slice::from_raw_parts_mut(pBuffer.cast::<u8>(), *piSize);

            let read_len = with_port!(handle, |port| {
                port.read(iAddress, buffer)
            })?;
            *piSize = read_len;
        }

        Ok(())

    }
}

gentl_api! {
    pub fn GCWritePort(
        hPort: PORT_HANDLE,
        iAddress: u64,
        pBuffer: *const libc::c_void,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        unsafe {
            let handle = ModuleHandle::from_raw_manually_drop(hPort)?;
            let data = std::slice::from_raw_parts(pBuffer.cast::<u8>(), *piSize);

            let written_len = with_port!(handle, |port| {
                port.write(iAddress, data)?
            });
            *piSize = written_len
        }

        Ok(())
    }
}

gentl_api! {
    pub fn GCReadPortStacked(
        hPort: PORT_HANDLE,
        pEntries: *mut PORT_REGISTER_STACK_ENTRY,
        piNumEntries: *mut libc::size_t,
    ) -> GenTlResult<()> {
        unsafe {
            let handle = ModuleHandle::from_raw_manually_drop(hPort)?;

            let mut entries: Vec<_> = (0..*piNumEntries)
                .map(|i| {
                    let raw_ent = *pEntries.add(i);
                    (
                        raw_ent.Address,
                        slice::from_raw_parts_mut(raw_ent.pBuffer.cast::<u8>(), raw_ent.Size),
                    )
                })
                .collect();

            with_port!(handle, |port| {
                port.read_stacked(&mut entries, piNumEntries.as_mut().unwrap())
            })
        }
    }
}

gentl_api! {
    pub fn GCWritePortStacked(
        hPort: PORT_HANDLE,
        pEntries: *mut PORT_REGISTER_STACK_ENTRY,
        piNumEntries: *mut libc::size_t,
    ) -> GenTlResult<()> {
        unsafe {
            let handle = ModuleHandle::from_raw_manually_drop(hPort)?;

            let entries: Vec<_> = (0..*piNumEntries)
                .map(|i| {
                    let raw_ent = *pEntries.add(i);
                    (
                        raw_ent.Address,
                        slice::from_raw_parts(raw_ent.pBuffer.cast::<u8>(), raw_ent.Size),
                    )
                })
                .collect();

            with_port!(handle, |port| {
                port.write_stacked(&entries, piNumEntries.as_mut().unwrap())
            })
        }
    }
}
