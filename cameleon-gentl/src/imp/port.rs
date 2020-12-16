use cameleon::device::CompressionType;
use semver::Version;

use crate::GenTlResult;

pub(crate) trait Port {
    /// Reads a number of bytes from a given address from the Port. This is the global
    /// GenICam GenApi read access function for all ports implemented in the GenTL
    /// implementation.
    fn read(&self, address: u64, buf: &mut [u8]) -> GenTlResult<usize>;

    /// Writes a number of bytes at the given address to the Port. This is the global
    /// GenICam GenApi write access function for all ports implemented in the GenTL
    /// implementation.
    fn write(&mut self, address: u64, data: &[u8]) -> GenTlResult<usize>;

    /// Read multiple entries.
    /// See "6.3.6 Port Functions" of GenTl specification for details.
    fn read_stacked(
        &self,
        entries: &mut [(u64, &mut [u8])],
        read_count: &mut usize,
    ) -> GenTlResult<()> {
        *read_count = 0;
        for ent in entries {
            self.read(ent.0, ent.1)?;
            *read_count += 1;
        }
        Ok(())
    }

    /// Write to multiple entries.
    /// See "6.3.6 Port Functions" of GenTl specification for details.
    fn write_stacked(
        &mut self,
        entries: &[(u64, &[u8])],
        written_count: &mut usize,
    ) -> GenTlResult<()> {
        *written_count = 0;

        for ent in entries {
            self.write(ent.0, ent.1)?;
            *written_count += 1;
        }

        Ok(())
    }

    /// Get detailed port information.
    fn port_info(&self) -> GenTlResult<&PortInfo>;

    /// Get available xml infos of the port.
    fn xml_infos(&self) -> GenTlResult<&[XmlInfo]>;
}

#[derive(Clone)]
pub(crate) struct PortInfo {
    /// Unique ID of the module the port reference.
    pub(crate) id: String,

    /// Port vendor name.
    /// In case the underlying module has no explicit vendor the vendor of the
    /// GenTL Producer is to be used.
    /// In case of a Buffer or a Data Stream the GenTL Producer vendor and model are to be used.
    pub(crate) vendor: String,

    /// Port modle name.
    pub(crate) model: String,

    /// Transport layer technology that is supported in the module.
    pub(crate) tl_type: TlType,

    /// GenTL Module the port refers to.
    pub(crate) module_type: ModuleType,

    /// Endianness of the port's data.
    pub(crate) endianness: Endianness,

    /// Access right of the port.
    pub(crate) access: PortAccess,

    /// Version of the port.
    pub(crate) version: Version,

    /// Name of the port as referenced in the XML description.
    /// This name is used to connect this port to the nodemap instance of this module.
    pub(crate) port_name: String,
}

#[derive(Clone, Copy)]
pub(crate) enum TlType {
    /// Camera Link.
    CameraLink,

    /// Camera Link High Speed.
    CameraLinkHS,

    /// CoaXPress.
    CoaXPress,

    /// GigE Vision.
    GigEVision,

    /// USB3 Vision.
    USB3Vision,

    /// This type is only valid for the System module in case the different Interface modules with a single system are of different types.
    /// All other modules must be of a defined type.
    Mixed,
}

#[derive(Clone, Copy)]
pub(crate) enum ModuleType {
    /// System Module.
    System,

    /// Interface Module.
    Interface,

    /// Device Module.
    Device,

    /// DataStream Module.
    DataStream,

    /// Buffer Module.
    Buffer,

    /// Remote Device.
    RemoteDevice,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PortAccess {
    /// Read Only.
    RO,

    /// Write Only.
    WO,

    /// Read Write.
    RW,

    /// Not Available.
    NA,

    /// Not implemented.
    NI,
}

impl PortAccess {
    pub(crate) fn is_readable(self) -> bool {
        match self {
            Self::RO | Self::RW => true,
            _ => false,
        }
    }

    pub(crate) fn is_writable(self) -> bool {
        match self {
            Self::WO | Self::RW => true,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Endianness {
    /// Little Endian.
    LE,
    /// Big Endian.
    BE,
}

#[derive(Clone)]
pub(crate) struct XmlInfo {
    pub(crate) location: XmlLocation,
    pub(crate) schema_version: Version,
    pub(crate) file_version: Version,
    pub(crate) sha1_hash: Option<[u8; 20]>,
    pub(crate) compressed: CompressionType,
}

#[derive(Clone)]
pub(crate) enum XmlLocation {
    RegisterMap { address: u64, size: usize },
    LocalFile(std::path::PathBuf),
    Url(url::Url),
}
