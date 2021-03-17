use cameleon_impl::memory::{memory, prelude::*, register_map};
use const_format::formatcp;

use crate::imp::{
    device,
    genapi_common::{
        GENTL_VERSION_MAJOR, GENTL_VERSION_MINOR, SCHEME_MAJOR_VERSION, SCHEME_MINOR_VERSION,
        SCHEME_SUBMINOR_VERSION,
    },
    port,
};

use GenApiReg::{
    DeviceAccessStatus, DeviceID, DeviceModelName, DeviceSelector, DeviceSelectorMax,
    DeviceUpdateList, DeviceVendorName,
};

#[memory]
pub(super) struct Memory {
    genapi_reg: GenApiReg,
    genapi_xml: GenApiXml,
}

#[register_map(base=0, endianness=LE)]
pub(super) enum GenApiReg {
    /// Updates the internal list of the devices when non zero value is wrritten to this
    /// register.
    #[register(len = 4, access = WO, ty = u32)]
    DeviceUpdateList,

    /// Selector for the different devices on this interface.
    #[register(len = 4, access = RW, ty = u32)]
    DeviceSelector,

    /// Maximum value of the device selector.
    #[register(len = 4, access = RO, ty = u32)]
    DeviceSelectorMax,

    /// Interface wide unique identifier of the selected device.
    #[register(len = 64, access = RO, ty = String)]
    DeviceID,

    /// Name of the device vendor.
    #[register(len = 128, access = RO, ty = String)]
    DeviceVendorName,

    /// Name of the device model.
    #[register(len = 128, access = RO, ty = String)]
    DeviceModelName,

    /// Gives the device's access status at the moment of the last execution of the DeviceUpdateList command.
    #[register(len = 4, access = RO, ty = u32)]
    DeviceAccessStatus,
}

#[register_map(base=GENAPI_XML_ADDRESS, endianness=LE)]
pub(super) enum GenApiXml {
    #[register(len = GENAPI_XML_LENGTH, access = RO, ty = String)]
    Xml = GENAPI_XML,
}

pub(super) const MODEL_NAME: &str = "CameleonGenTLU3VInterfaceModule";
pub(super) const VENDOR_NAME: &str = "CameleonProjectDevelopers";
pub(super) const TOOL_TIP: &str = "GenTL U3V Interface Module";

pub(super) const INTERFACE_ID: &str = PRODUCT_GUID;
pub(super) const INTERFACE_TYPE: port::TlType = port::TlType::USB3Vision;
pub(super) const PORT_NAME: &str = "InterfacePort";

pub(super) const XML_MAJOR_VERSION: u64 = 1;
pub(super) const XML_MINOR_VERSION: u64 = 0;
pub(super) const XML_SUBMINOR_VERSION: u64 = 0;

pub(super) const GENAPI_XML_ADDRESS: usize = GenApiReg::base() + GenApiReg::size();
pub(super) const GENAPI_XML_LENGTH: usize = GENAPI_XML.len();

const PRODUCT_GUID: &str = "639290f8-043c-436d-b8d1-cb916e2928e9";
const VERSION_GUID: &str = "fee92fd7-f388-44c7-bd07-c7f706bdc182";

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
        <pFeature>InterfaceInformation</pFeature>
        <pFeature>DeviceEnumeration</pFeature>
    </Category>

    <Port Name="{PORT_NAME}" NameSpace="Standard">
        <Description>The GenICam port through which the Interface module is accessed.</Description>
        <Visibility>Invisible</Visibility>
    </Port>

    <Category Name="InterfaceInformation" NameSpace="Standard">
        <Description>Category that contains all Interface Information features of the Interface module.</Description>
        <Visibility>Beginner</Visibility>
        <pFeature>InterfaceID</pFeature>
        <pFeature>InterfaceType</pFeature>
        <pFeature>InterfaceTLVersionMajor</pFeature>
        <pFeature>InterfaceTLVersionMinor</pFeature>
    </Category>

    <String Name="InterfaceID" NameSpace="Standard">
        <Description>GenTL Producer wide unique identifier of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <Value>{INTERFACE_ID}</Value>
    </String>

    <Enumeration Name="InterfaceType" NameSpace="Standard">
        <Description>Transport layer type of the interface.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="{interface_type}" NameSpace="Standard">
            <Description>USB3 Vision</Description>
            <Value>0</Value>
        </EnumEntry>
        <Value>0</Value>
    </Enumeration>

    <Integer Name="InterfaceTLVersionMajor" NameSpace="Standard">
        <Description>Major version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>{GENTL_VERSION_MAJOR}</Value>
        <Min>{GENTL_VERSION_MAJOR}</Min>
        <Max>{GENTL_VERSION_MAJOR}</Max>
    </Integer>

    <Integer Name="InterfaceTLVersionMinor" NameSpace="Standard">
        <Description>Minor version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>{GENTL_VERSION_MINOR}</Value>
        <Min>{GENTL_VERSION_MINOR}</Min>
        <Max>{GENTL_VERSION_MINOR}</Max>
    </Integer>

    <Category Name="DeviceEnumeration" NameSpace="Standard">
        <Description>Category that contains all Device Enumeration features of the Interface module.</Description>
        <Visibility>Expert</Visibility>
        <pFeature>DeviceUpdateList</pFeature>
        <pFeature>DeviceSelector</pFeature>
        <pFeature>DeviceID</pFeature>
        <pFeature>DeviceVendorName</pFeature>
        <pFeature>DeviceModelName</pFeature>
        <pFeature>DeviceAccessStatus</pFeature>
        <pFeature>DeviceTLVersionMajor</pFeature>
        <pFeature>DeviceTLVersionMinor</pFeature>
    </Category>

    <Command Name="DeviceUpdateList" NameSpace="Standard">
        <Description>Updates the internal list of the devices.</Description>
        <Visibility>Expert</Visibility>
        <ImposedAccessMode>WO</ImposedAccessMode>
        <pValue>DeviceUpdateListReg</pValue>
        <CommandValue>1</CommandValue>
    </Command>

    <IntReg Name="DeviceUpdateListReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{device_update_list_addr}</Address>
        <Length>{device_update_list_len}</Length>
        <AccessMode>{device_update_list_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <Integer Name="DeviceSelector" NameSpace="Standard">
        <Description>Selector for the different devices on this interface.</Description>
        <Visibility>Expert</Visibility>
        <pValue>DeviceSelectorReg</pValue>
        <Min>0</Min>
        <pMax>DeviceSelectorMaxReg</pMax>
    </Integer>

    <IntReg Name="DeviceSelectorReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{device_selector_addr}</Address>
        <Length>{device_selector_len}</Length>
        <AccessMode>{device_selector_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="DeviceSelectorMaxReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{device_selector_max_addr}</Address>
        <Length>{device_selector_max_len}</Length>
        <AccessMode>{device_selector_max_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <StringReg Name="DeviceID" NameSpace="Standard">
        <Description>Interface wide unique identifier of the selected device.</Description>
        <Visibility>Expert</Visibility>
        <Address>{device_id_addr}</Address>
        <Length>{device_id_len}</Length>
        <AccessMode>{device_id_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <StringReg Name="DeviceVendorName" NameSpace="Standard">
        <Description>Name of the device vendor.</Description>
        <Visibility>Expert</Visibility>
        <Address>{device_vendor_name_addr}</Address>
        <Length>{device_vendor_name_len}</Length>
        <AccessMode>{device_vendor_name_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <StringReg Name="DeviceModelName" NameSpace="Standard">
        <Description>Name of the device model.</Description>
        <Visibility>Expert</Visibility>
        <Address>{device_model_name_addr}</Address>
        <Length>{device_model_name_len}</Length>
        <AccessMode>{device_model_name_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <Enumeration Name="DeviceAccessStatus" NameSpace="Standard">
        <Description>Gives the device's access status at the moment of the last execution of the DeviceUpdateList command.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="{device_access_status_unknown_str}" NameSpace="Standard">
            <Description>Not known to producer.</Description>
            <Value>{device_access_status_unknown_int}</Value>
        </EnumEntry>

        <EnumEntry Name="{device_access_status_readwrite_str}" NameSpace="Standard">
            <Description>Full access.</Description>
            <Value>{device_access_status_readwrite_int}</Value>
        </EnumEntry>

        <EnumEntry Name="{device_access_status_readonly_str}" NameSpace="Standard">
            <Description>Read-only access.</Description>
            <Value>{device_access_status_readonly_int}</Value>
        </EnumEntry>

        <EnumEntry Name="{device_access_status_noaccess_str}" NameSpace="Standard">
            <Description>Not available to connect.</Description>
            <Value>{device_access_status_noaccess_int}</Value>
        </EnumEntry>

        <EnumEntry Name="{device_access_status_busy_str}" NameSpace="Standard">
            <Description>The device is already opened by another entity.</Description>
            <Value>{device_access_status_busy_int}</Value>
        </EnumEntry>

        <EnumEntry Name="{device_access_status_openrw_str}" NameSpace="Standard">
            <Description>Open in Read/Write mode by this GenTL host.</Description>
            <Value>{device_access_status_openrw_int}</Value>
        </EnumEntry>

        <EnumEntry Name="{device_access_status_openro_str}" NameSpace="Standard">
            <Description>Open in Read only mode by this GenTL host</Description>
            <Value>{device_access_status_openro_int}</Value>
        </EnumEntry>

        <pValue>DeviceAccessStatusReg</pValue>
    </Enumeration>

    <IntReg Name="DeviceAccessStatusReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{device_access_status_addr}</Address>
        <Length>{device_access_status_len}</Length>
        <AccessMode>{device_access_status_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <Integer Name="DeviceTLVersionMajor" NameSpace="Standard">
        <Description>Major version number of the transport layer specification the remote device complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>{GENTL_VERSION_MAJOR}</Value>
        <Min>{GENTL_VERSION_MAJOR}</Min>
        <Max>{GENTL_VERSION_MAJOR}</Max>
    </Integer>

    <Integer Name="DeviceTLVersionMinor" NameSpace="Standard">
        <Description>Minor version number of the transport layer specification the remote device complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>{GENTL_VERSION_MINOR}</Value>
        <Min>{GENTL_VERSION_MINOR}</Min>
        <Max>{GENTL_VERSION_MINOR}</Max>
    </Integer>
</RegisterDescription>"#,
    interface_type = INTERFACE_TYPE.as_str(),
    device_update_list_addr = DeviceUpdateList::ADDRESS,
    device_update_list_len = DeviceUpdateList::LENGTH,
    device_update_list_access = DeviceUpdateList::ACCESS_RIGHT.as_str(),
    device_selector_addr = DeviceSelector::ADDRESS,
    device_selector_len = DeviceSelector::LENGTH,
    device_selector_access = DeviceSelector::ACCESS_RIGHT.as_str(),
    device_selector_max_addr = DeviceSelectorMax::ADDRESS,
    device_selector_max_len = DeviceSelectorMax::LENGTH,
    device_selector_max_access = DeviceSelectorMax::ACCESS_RIGHT.as_str(),
    device_id_addr = DeviceID::ADDRESS,
    device_id_len = DeviceID::LENGTH,
    device_id_access = DeviceID::ACCESS_RIGHT.as_str(),
    device_vendor_name_addr = DeviceVendorName::ADDRESS,
    device_vendor_name_len = DeviceVendorName::LENGTH,
    device_vendor_name_access = DeviceVendorName::ACCESS_RIGHT.as_str(),
    device_model_name_addr = DeviceModelName::ADDRESS,
    device_model_name_len = DeviceModelName::LENGTH,
    device_model_name_access = DeviceModelName::ACCESS_RIGHT.as_str(),
    device_access_status_unknown_str = device::DeviceAccessStatus::Unknown.as_str(),
    device_access_status_unknown_int = device::DeviceAccessStatus::Unknown as i32,
    device_access_status_readwrite_str = device::DeviceAccessStatus::ReadWrite.as_str(),
    device_access_status_readwrite_int = device::DeviceAccessStatus::ReadWrite as i32,
    device_access_status_readonly_str = device::DeviceAccessStatus::ReadOnly.as_str(),
    device_access_status_readonly_int = device::DeviceAccessStatus::ReadOnly as i32,
    device_access_status_noaccess_str = device::DeviceAccessStatus::NoAccess.as_str(),
    device_access_status_noaccess_int = device::DeviceAccessStatus::NoAccess as i32,
    device_access_status_busy_str = device::DeviceAccessStatus::Busy.as_str(),
    device_access_status_busy_int = device::DeviceAccessStatus::Busy as i32,
    device_access_status_openrw_str = device::DeviceAccessStatus::OpenReadWrite.as_str(),
    device_access_status_openrw_int = device::DeviceAccessStatus::OpenReadWrite as i32,
    device_access_status_openro_str = device::DeviceAccessStatus::OpenReadOnly.as_str(),
    device_access_status_openro_int = device::DeviceAccessStatus::OpenReadOnly as i32,
    device_access_status_addr = DeviceAccessStatus::ADDRESS,
    device_access_status_len = DeviceAccessStatus::LENGTH,
    device_access_status_access = DeviceAccessStatus::ACCESS_RIGHT.as_str(),
);
