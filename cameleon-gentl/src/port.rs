use cameleon_impl::memory::MemoryError;
use semver::Version;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortError {
    /// The handle isn't opend.
    #[error("the handle isn't opend")]
    NotOpend,

    /// The access to the requested register address is denied because the register is not writable
    /// or because the Port module is opened in a way that it does not allow write access.
    #[error("the access to the requested register addresss is denied")]
    AccessDenied,

    /// There is no register with the provided address.
    #[error("there is no register with the provided address")]
    InvalidAddress,

    /// An invalid value has been written.
    #[error("An invalid value has been written: {}", 0)]
    InvalidValue(std::borrow::Cow<'static, str>),

    /// Communication error or connection lost.
    #[error("Communication error or connection lost: {}", 0)]
    IoError(Box<dyn std::error::Error>),
}

impl From<MemoryError> for PortError {
    fn from(err: MemoryError) -> Self {
        match err {
            MemoryError::AddressNotReadable | MemoryError::AddressNotWritable => Self::AccessDenied,
            MemoryError::InvalidAddress => Self::InvalidAddress,
            MemoryError::InvalidRegisterData(cause) => Self::InvalidValue(cause),
        }
    }
}

pub type PortResult<T> = std::result::Result<T, PortError>;

pub trait Port {
    /// Reads a number of bytes from a given address from the Port. This is the global
    /// GenICam GenApi read access function for all ports implemented in the GenTL
    /// implementation.
    fn read(&self, address: u64, size: usize) -> PortResult<Vec<u8>>;

    /// Writes a number of bytes at the given address to the Port. This is the global
    /// GenICam GenApi write access function for all ports implemented in the GenTL
    /// implementation.
    fn write(&self, address: u64, data: &[u8]) -> PortResult<usize>;

    /// Get detailed port information.
    fn port_info(&self) -> PortResult<PortInfo>;

    /// Get available xml infos of the port.
    fn xml_infos(&self) -> PortResult<Vec<XmlInfo>>;
}

pub struct PortInfo {
    /// Unique ID of the module the port reference.
    pub id: String,

    /// Port vendor name.
    /// In case the underlying module has no explicit vendor the vendor of the
    /// GenTL Producer is to be used.
    /// In case of a Buffer or a Data Stream the GenTL Producer vendor and model are to be used.
    pub vendor: String,

    /// Transport layer technology that is supported in the module.
    pub tl_type: TLType,

    /// GenTL Module the port refers to.
    pub module_type: ModuleType,

    /// Endianness of the port's data.
    pub endianness: Endianness,

    /// Access right of the port.
    pub access: PortAccess,

    /// Version of the port.
    pub version: Version,

    /// Name of the port as referenced in the XML description.
    /// This name is used to connect this port to the nodemap instance of this module.
    pub port_name: String,
}

pub enum TLType {
    /// GigE Vision.
    GEV,

    /// Camera Link.
    CL,

    /// IIDC 1394.
    IIDC,

    /// USB video class.
    UVC,

    /// CoaXPress.
    CXP,

    /// Camera Link HS.
    CLHS,

    /// USB3 Vision Standard.
    U3V,

    /// Generic Ethernet.
    Ethernet,

    /// PCI/PCIe
    PCI,

    /// This type is only valid for the System module in case the different Interface modules with a single system are of different types.
    /// All other modules must be of a defined type.
    Mixed,
}

pub enum ModuleType {
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

pub enum PortAccess {
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

pub enum Endianness {
    /// Little Endian.
    LE,
    /// Big Endian.
    BE,
}

pub struct XmlInfo {
    pub location: XmlLocation,
    pub schema_version: Version,
    pub compressed: Compressed,
}

pub enum Compressed {
    None,
    Zip,
}

pub enum XmlLocation {
    RegisterMap { address: u64, size: usize },
    LocalFile(std::path::PathBuf),
    Url(url::Url),
}
