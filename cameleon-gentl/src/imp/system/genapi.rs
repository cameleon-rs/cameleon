use cameleon_impl::memory::{memory, prelude::*, register_map};
use const_format::formatcp;

use crate::imp::{
    genapi_common::{
        GENTL_VERSION_MAJOR, GENTL_VERSION_MINOR, SCHEME_MAJOR_VERSION, SCHEME_MINOR_VERSION,
        SCHEME_SUBMINOR_VERSION,
    },
    port,
};

use GenApiReg::{
    GevInterfaceDefaultGateway, GevInterfaceDefaultIPAddress, GevInterfaceDefaultSubnetMask,
    GevInterfaceMACAddress, InterfaceID, InterfaceSelector, InterfaceSelectorMax,
    InterfaceUpdateList, TlPath,
};

#[memory]
pub(super) struct Memory {
    genapi_reg: GenApiReg,
    genapi_xml: GenApiXml,
}

#[register_map(base=0, endianness=LE)]
pub(super) enum GenApiReg {
    /// Full path to the GenTL producer.
    #[register(len = 1024, access = RO, ty = String)]
    TlPath,

    /// Updates the internal list of the interfaces when non zero value is wrritten to this
    /// register.
    #[register(len = 4, access = RO, ty = u32)]
    InterfaceUpdateList,

    /// Selector for the different GenTL Producer interfaces.
    #[register(len = 4, access = RW, ty = u32)]
    InterfaceSelector,

    /// Maximum value of interface selector.
    #[register(len = 4, access = RO, ty = u32)]
    InterfaceSelectorMax,

    /// GenTL Producer wide unique identifier of the selected interface.
    #[register(len = 64, access = RO, ty = String)]
    InterfaceID,

    /// 48-bit MAC address of the selected interface.
    #[register(len = 8, access = RO, ty = BitField<u64, LSB = 0, MSB = 47>)]
    GevInterfaceMACAddress,

    /// IP address of the first subnet of the selected interface.
    #[register(len = 4, access = RO, ty = u32)]
    GevInterfaceDefaultIPAddress,

    /// Subnet mask of the first subnet of the selected interface.
    #[register(len = 4, access = RO, ty = u32)]
    GevInterfaceDefaultSubnetMask,

    /// Gateway of the selected interface.
    #[register(len = 4, access = RO, ty = u32)]
    GevInterfaceDefaultGateway,
}

#[register_map(base=GENAPI_XML_ADDRESS, endianness=LE)]
pub(super) enum GenApiXml {
    #[register(len = GENAPI_XML_LENGTH, access = RO, ty = String)]
    Xml = GENAPI_XML,
}

pub(super) const MODEL_NAME: &str = "CameleonGenTLSystemModule";
pub(super) const VENDOR_NAME: &str = "CameleonProjectDevelopers";
pub(super) const TOOL_TIP: &str = "GenTL System Module";
pub(super) const TLID: &str = PRODUCT_GUID;

pub(super) const XML_MAJOR_VERSION: u64 = 1;
pub(super) const XML_MINOR_VERSION: u64 = 0;
pub(super) const XML_SUBMINOR_VERSION: u64 = 0;

pub(super) const PORT_NAME: &str = "TLPort";

pub(super) const GENAPI_XML_LENGTH: usize = GENAPI_XML.len();
pub(super) const GENAPI_XML_ADDRESS: usize = GenApiReg::base() + GenApiReg::size();
pub(super) const TL_TYPE: port::TlType = port::TlType::Mixed;

const PRODUCT_GUID: &str = "C09F0257-3F5C-41C2-B34F-FE67CB108370";
const VERSION_GUID: &str = "10F7AF60-A1B0-4AE4-8785-F214C22DAA9D";

const GENAPI_XML: &str = formatcp!(
    r#"<?xml version="1.0" encoding="UTF-8"?>
<RegisterDescription
ModelName="{MODEL_NAME}"
VendorName="{VENDOR_NAME}"
StandardNameSpace="None"
SchemaMajorVersion="{SCHEME_MAJOR_VERSION}"
SchemaMinorVersion="{SCHEME_MINOR_VERSION}"
SchemaSubMinorVersion="{SCHEME_SUBMINOR_VERSION}"
MajorVersion="{XML_MAJOR_VERSION}"
MinorVersion="{XML_MINOR_VERSION}"
SubMinorVersion="{XML_SUBMINOR_VERSION}"
ToolTip="{TOOL_TIP}"
ProductGuid="{PRODUCT_GUID}"
VersionGuid="{VERSION_GUID}"
xmlns="http://www.genicam.org/GenApi/Version_1_1"
xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
xsi:schemaLocation="http://www.genicam.org/GenApi/Version_1_1 http://www.genicam.org/GenApi/GenApiSchema_Version_1_1.xsd">

    <Category Name="Root" NameSpace="Standard">
        <Description>Provides the Root of the GenICam features tree.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>SystemInformation</pFeature>
        <pFeature>InterfaceEnumeration</pFeature>
    </Category>

    <Port Name="{PORT_NAME}" NameSpace="Standard">
        <Description>The GenICam port through which the System module is accessed.</Description>
        <Visibility>Invisible</Visibility>
    </Port>

    <Category Name="SystemInformation" NameSpace="Standard">
        <Description>Category that contains all System Information features of the System module.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>TLID</pFeature>
        <pFeature>TLVendorName</pFeature>
        <pFeature>TLModelName</pFeature>
        <pFeature>TLVersion</pFeature>
        <pFeature>TLPath</pFeature>
        <pFeature>TLType</pFeature>
        <pFeature>GenTLVersionMajor</pFeature>
        <pFeature>GenTLVersionMinor</pFeature>
    </Category>

    <String Name="TLID" NameSpace="Standard">
        <Description>Unique identifier of the GenTL Producer like a GUID.</Description>
        <Visibility>Expert</Visibility>

        <Value>{TLID}</Value>
    </String>

    <String Name="TLVendorName" NameSpace="Standard">
        <Description>Name of the GenTL Producer vendor.</Description>
        <Visibility>Beginner</Visibility>

        <Value>{VENDOR_NAME}</Value>
    </String>

    <String Name="TLModelName" NameSpace="Standard">
        <Description>Name of the GenTL Producer to distinguish different kinds of GenTL Producer implementations from one vendor.</Description>
        <Visibility>Beginner</Visibility>

        <Value>{MODEL_NAME}</Value>
    </String>

    <String Name="TLVersion" NameSpace="Standard">
        <Description>Vendor specific version string of the GenTL Producer.</Description>
        <Visibility>Beginner</Visibility>

        <Value>{XML_MAJOR_VERSION}.{XML_MINOR_VERSION}.{XML_SUBMINOR_VERSION}</Value>
    </String>

    <StringReg Name="TLPath" NameSpace="Standard">
        <Description>Full path to the GenTL Producer including filename and extension.</Description>
        <Visibility>Beginner</Visibility>

        <Address>{tl_path_addr}</Address>
        <Length>{tl_path_length}</Length>
        <AccessMode>{tl_path_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <Enumeration Name="TLType" NameSpace="Standard">
        <Description>Transport layer type of the GenTL Producer implementation.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="{tl_type}" NameSpace="Standard">
            <Description>Different Interface modules of the GenTL Producer are of different types.</Description>
            <Value>0</Value>
        </EnumEntry>
        <Value>0</Value>
    </Enumeration>

    <Integer Name="GenTLVersionMajor" NameSpace="Standard">
        <Description>Major version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>{GENTL_VERSION_MAJOR}</Value>
        <Min>{GENTL_VERSION_MAJOR}</Min>
        <Max>{GENTL_VERSION_MAJOR}</Max>
    </Integer>

    <Integer Name="GenTLVersionMinor" NameSpace="Standard">
        <Description>Minor version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>{GENTL_VERSION_MINOR}</Value>
        <Min>{GENTL_VERSION_MINOR}</Min>
        <Max>{GENTL_VERSION_MINOR}</Max>
    </Integer>

    <Integer Name="GenTLSFNCVersionMajor" NameSpace="Standard">
        <Description>Major version number of the GenTL Standard Features Naming Convention that was used to create the GenTL Producer`s XML.</Description>
        <Visibility>Expert</Visibility>
        <Value>1</Value>
        <Min>1</Min>
        <Max>1</Max>
    </Integer>

    <Category Name="InterfaceEnumeration" NameSpace="Standard">
        <Description>Category that contains all Interface Enumeration features of the System module.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>InterfaceUpdateList</pFeature>
        <pFeature>InterfaceSelector</pFeature>
        <pFeature>InterfaceID</pFeature>
        <pFeature>GevInterfaceMACAddress</pFeature>
        <pFeature>GevInterfaceDefaultIPAddress</pFeature>
        <pFeature>GevInterfaceDefaultSubnetMask</pFeature>
        <pFeature>GevInterfaceDefaultGateway</pFeature>
    </Category>

    <Command Name="InterfaceUpdateList" NameSpace="Standard">
        <Description>Updates the internal list of the interfaces.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>InterfaceUpdateListReg</pValue>
        <CommandValue>1</CommandValue>
    </Command>

    <IntReg Name="InterfaceUpdateListReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{update_list_addr}</Address>
        <Length>{update_list_len}</Length>
        <AccessMode>{update_list_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <Integer Name="InterfaceSelector" NameSpace="Standard">
        <Description>Selector for the different GenTL Producer interfaces.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>InterfaceSelectorReg</pValue>
        <Min>0</Min>
        <pMax>InterfaceSelectorMaxReg</pMax>
    </Integer>

    <IntReg Name="InterfaceSelectorReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{interface_selector_addr}</Address>
        <Length>{interface_selector_len}</Length>
        <AccessMode>{interface_selector_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="InterfaceSelectorMaxReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{interface_selector_max_addr}</Address>
        <Length>{interface_selector_max_len}</Length>
        <AccessMode>{interface_selector_max_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <StringReg Name="InterfaceID" NameSpace="Standard">
        <Description>GenTL Producer wide unique identifier of the selected interface.</Description>
        <Visibility>Beginner</Visibility>
        <Address>{interface_id_addr}</Address>
        <Length>{interface_id_len}</Length>
        <AccessMode>{interface_id_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <MaskedIntReg Name="GevInterfaceMACAddress" NameSpace="Standard">
        <Description>48-bit MAC address of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <Address>{mac_address_addr}</Address>
        <Length>{mac_address_len}</Length>
        <AccessMode>{mac_address_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <LSB>{mac_address_lsb}</LSB>
        <MSB>{mac_address_msb}</MSB>
        <Endianess>LittleEndian</Endianess>
        <Representation>MACAddress</Representation>
    </MaskedIntReg>

    <IntReg Name="GevInterfaceDefaultIPAddress" NameSpace="Standard">
        <Description>IP address of the first subnet of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <Address>{default_ip_address_addr}</Address>
        <Length>{default_ip_address_len}</Length>
        <AccessMode>{default_ip_address_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
        <Representation>IPV4Address</Representation>
    </IntReg>

    <IntReg Name="GevInterfaceDefaultSubnetMask" NameSpace="Standard">
        <Description>Subnet mask of the first subnet of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <Address>{default_subnetmask_address_addr}</Address>
        <Length>{default_subnetmask_address_len}</Length>
        <AccessMode>{default_subnetmask_address_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
        <Representation>IPV4Address</Representation>
    </IntReg>

    <IntReg Name="GevInterfaceDefaultGateway" NameSpace="Standard">
        <Description>Gateway of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <Address>{default_gateway_addr}</Address>
        <Length>{default_gateway_len}</Length>
        <AccessMode>{default_gateway_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
        <Representation>IPV4Address</Representation>
    </IntReg>
</RegisterDescription>"#,
    tl_type = TL_TYPE.as_str(),
    tl_path_addr = TlPath::ADDRESS,
    tl_path_length = TlPath::LENGTH,
    tl_path_access = TlPath::ACCESS_RIGHT.as_str(),
    update_list_addr = InterfaceUpdateList::ADDRESS,
    update_list_len = InterfaceUpdateList::LENGTH,
    update_list_access = InterfaceUpdateList::ACCESS_RIGHT.as_str(),
    interface_selector_addr = InterfaceSelector::ADDRESS,
    interface_selector_len = InterfaceSelector::LENGTH,
    interface_selector_access = InterfaceSelector::ACCESS_RIGHT.as_str(),
    interface_selector_max_addr = InterfaceSelectorMax::ADDRESS,
    interface_selector_max_len = InterfaceSelectorMax::LENGTH,
    interface_selector_max_access = InterfaceSelectorMax::ACCESS_RIGHT.as_str(),
    interface_id_addr = InterfaceID::ADDRESS,
    interface_id_len = InterfaceID::LENGTH,
    interface_id_access = InterfaceID::ACCESS_RIGHT.as_str(),
    mac_address_addr = GevInterfaceMACAddress::ADDRESS,
    mac_address_len = GevInterfaceMACAddress::LENGTH,
    mac_address_access = GevInterfaceMACAddress::ACCESS_RIGHT.as_str(),
    mac_address_lsb = GevInterfaceMACAddress::LSB,
    mac_address_msb = GevInterfaceMACAddress::MSB,
    default_ip_address_addr = GevInterfaceDefaultIPAddress::ADDRESS,
    default_ip_address_len = GevInterfaceDefaultIPAddress::LENGTH,
    default_ip_address_access = GevInterfaceDefaultIPAddress::ACCESS_RIGHT.as_str(),
    default_subnetmask_address_addr = GevInterfaceDefaultSubnetMask::ADDRESS,
    default_subnetmask_address_len = GevInterfaceDefaultSubnetMask::LENGTH,
    default_subnetmask_address_access = GevInterfaceDefaultSubnetMask::ACCESS_RIGHT.as_str(),
    default_gateway_addr = GevInterfaceDefaultGateway::ADDRESS,
    default_gateway_len = GevInterfaceDefaultGateway::LENGTH,
    default_gateway_access = GevInterfaceDefaultGateway::ACCESS_RIGHT.as_str(),
);
