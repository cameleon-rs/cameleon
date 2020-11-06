use semver::Version;

use super::GenTlResult;

pub trait Port {
    /// Reads a number of bytes from a given address from the Port. This is the global
    /// GenICam GenApi read access function for all ports implemented in the GenTL
    /// implementation.
    fn read(&self, address: u64, size: usize) -> GenTlResult<Vec<u8>>;

    /// Writes a number of bytes at the given address to the Port. This is the global
    /// GenICam GenApi write access function for all ports implemented in the GenTL
    /// implementation.
    fn write(&mut self, address: u64, data: &[u8]) -> GenTlResult<()>;

    /// Get detailed port information.
    fn port_info(&self) -> &PortInfo;

    /// Get available xml infos of the port.
    fn xml_infos(&self) -> &[XmlInfo];
}

#[derive(Clone)]
pub struct PortInfo {
    /// Unique ID of the module the port reference.
    pub id: String,

    /// Port vendor name.
    /// In case the underlying module has no explicit vendor the vendor of the
    /// GenTL Producer is to be used.
    /// In case of a Buffer or a Data Stream the GenTL Producer vendor and model are to be used.
    pub vendor: String,

    /// Transport layer technology that is supported in the module.
    pub tl_type: TlType,

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

#[derive(Clone)]
pub enum TlType {
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

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub enum Endianness {
    /// Little Endian.
    LE,
    /// Big Endian.
    BE,
}

#[derive(Clone)]
pub struct XmlInfo {
    pub location: XmlLocation,
    pub schema_version: Version,
    pub compressed: Compressed,
}

#[derive(Clone)]
pub enum Compressed {
    None,
    Zip,
}

#[derive(Clone)]
pub enum XmlLocation {
    RegisterMap { address: u64, size: usize },
    LocalFile(std::path::PathBuf),
    Url(url::Url),
}
