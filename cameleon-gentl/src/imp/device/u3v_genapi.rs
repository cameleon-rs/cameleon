use cameleon_impl::memory::{memory, prelude::*, register_map};
use const_format::formatcp;

use GenApiReg::*;

use crate::imp::{genapi_common::*, port};

#[memory]
pub(super) struct Memory {
    genapi_reg: GenApiReg,
    genapi_xml: GenApiXml,
}

#[register_map(base=0, endianness=LE)]
pub(super) enum GenApiReg {
    /// Interface wide unique identifier of the selected device.
    #[register(len = 64, access = RO, ty = String)]
    DeviceID,

    /// Vendor name of the remote device.
    #[register(len = 128, access = RO, ty = String)]
    DeviceVendorName,

    /// Model name of the remote device.
    #[register(len = 128, access = RO, ty = String)]
    DeviceModelName,

    /// Gives the device's access status at the moment of the last execution of the DeviceUpdateList command.
    #[register(len = 4, access = RO, ty = u32)]
    DeviceAccessStatus,

    /// Selector for the different stream channels.
    #[register(len = 4, access = RW, ty = u32)]
    StreamSelector,

    /// Maximum value of the stream selector.
    #[register(len = 4, access = RO, ty = u32)]
    StreamSelectorMax,

    /// Interface wide unique identifier of the selected stream.
    #[register(len = 64, access = RO, ty = String)]
    StreamID,
}

#[register_map(base=0, endianness=LE)]
pub(super) enum GenApiXml {
    #[register(len = GENAPI_XML_LENGTH, access = RO, ty = String)]
    Xml = GENAPI_XML,
}

pub(super) const MODEL_NAME: &str = "CameleonGenTLU3VDeviceModule";
pub(super) const VENDOR_NAME: &str = "CameleonProjectDevelopers";
pub(super) const TOOL_TIP: &str = "GenTL U3V Device Module";

pub(super) const DEVICE_TYPE: port::TlType = port::TlType::USB3Vision;
pub(super) const PORT_NAME: &str = "DevicePort";

const PRODUCT_GUID: &str = "21fbd4d2-6244-445b-827f-6fd92a8787c8";
const VERSION_GUID: &str = "0cd40ca9-9db8-49db-9952-cb758555d04e";

pub(super) const XML_MAJOR_VERSION: u64 = 1;
pub(super) const XML_MINOR_VERSION: u64 = 0;
pub(super) const XML_SUBMINOR_VERSION: u64 = 0;

pub(super) const GENAPI_XML_ADDRESS: usize = GenApiReg::base() + GenApiReg::size();
pub(super) const GENAPI_XML_LENGTH: usize = GENAPI_XML.len();

const GENAPI_XML: &str = formatcp!(
    r#"<RegisterDescription
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
        <pFeature>DeviceInformation</pFeature>
        <pFeature>StreamEnumeration</pFeature>
    </Category>

    <Port Name="{PORT_NAME}" NameSpace="Standard">
        <Description>The GenICam port through which the Device module is accessed.</Description>
        <Visibility>Invisible</Visibility>
    </Port>

    <Category Name="DeviceInformation" NameSpace="Standard">
        <Description>Category that contains all Device Information features of the Device module.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>DeviceID</pFeature>
        <pFeature>DeviceVendorName</pFeature>
        <pFeature>DeviceModelName</pFeature>
        <pFeature>DeviceType</pFeature>
        <pFeature>DeviceAccessStatus</pFeature>
    </Category>

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
        <Visibility>Beginner</Visibility>
        <Address>{device_vendor_name_addr}</Address>
        <Length>{device_vendor_name_len}</Length>
        <AccessMode>{device_vendor_name_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <StringReg Name="DeviceModelName" NameSpace="Standard">
        <Description>Name of the device model.</Description>
        <Visibility>Beginner</Visibility>
        <Address>{device_model_name_addr}</Address>
        <Length>{device_model_name_len}</Length>
        <AccessMode>{device_model_name_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

    <Enumeration Name="DeviceType" NameSpace="Standard">
        <Description>Transport layer type of the device.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="{device_type}" NameSpace="Standard">
            <Description>USB3 Vision</Description>
            <Value>0</Value>
        </EnumEntry>
        <Value>0</Value>
    </Enumeration>

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

    <Category Name="StreamEnumeration" NameSpace="Standard">
        <Description>Category that contains all Stream Enumeration features of the Device module.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>StreamSelector</pFeature>
        <pFeature>StreamID</pFeature>
    </Category>

    <Integer Name="StreamSelector" NameSpace="Standard">
        <Description>Selector for the different stream channels.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>StreamSelectorReg</pValue>
        <Min>0</Min>
        <pMax>StreamSelectorMaxReg</pMax>
    </Integer>

    <IntReg Name="StreamSelectorReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{stream_selector_addr}</Address>
        <Length>{stream_selector_len}</Length>
        <AccessMode>{stream_selector_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="StreamSelectorMaxReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>{stream_selector_max_addr}</Address>
        <Length>{stream_selector_max_len}</Length>
        <AccessMode>{stream_selector_max_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <StringReg Name="StreamID" NameSpace="Standard">
        <Description>Device unique ID for the stream.</Description>
        <Visibility>Expert</Visibility>
        <Address>{stream_id_addr}</Address>
        <Length>{stream_id_len}</Length>
        <AccessMode>{stream_id_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
    </StringReg>

</RegisterDescription>"#,
    device_id_addr = DeviceID::ADDRESS,
    device_id_len = DeviceID::LENGTH,
    device_id_access = DeviceID::ACCESS_RIGHT.as_str(),
    device_vendor_name_addr = DeviceVendorName::ADDRESS,
    device_vendor_name_len = DeviceVendorName::LENGTH,
    device_vendor_name_access = DeviceVendorName::ACCESS_RIGHT.as_str(),
    device_model_name_addr = DeviceModelName::ADDRESS,
    device_model_name_len = DeviceModelName::LENGTH,
    device_model_name_access = DeviceModelName::ACCESS_RIGHT.as_str(),
    device_type = DEVICE_TYPE.as_str(),
    device_access_status_unknown_str = super::DeviceAccessStatus::Unknown.as_str(),
    device_access_status_unknown_int = super::DeviceAccessStatus::Unknown as i32,
    device_access_status_readwrite_str = super::DeviceAccessStatus::ReadWrite.as_str(),
    device_access_status_readwrite_int = super::DeviceAccessStatus::ReadWrite as i32,
    device_access_status_readonly_str = super::DeviceAccessStatus::ReadOnly.as_str(),
    device_access_status_readonly_int = super::DeviceAccessStatus::ReadOnly as i32,
    device_access_status_noaccess_str = super::DeviceAccessStatus::NoAccess.as_str(),
    device_access_status_noaccess_int = super::DeviceAccessStatus::NoAccess as i32,
    device_access_status_busy_str = super::DeviceAccessStatus::Busy.as_str(),
    device_access_status_busy_int = super::DeviceAccessStatus::Busy as i32,
    device_access_status_openrw_str = super::DeviceAccessStatus::OpenReadWrite.as_str(),
    device_access_status_openrw_int = super::DeviceAccessStatus::OpenReadWrite as i32,
    device_access_status_openro_str = super::DeviceAccessStatus::OpenReadOnly.as_str(),
    device_access_status_openro_int = super::DeviceAccessStatus::OpenReadOnly as i32,
    device_access_status_addr = DeviceAccessStatus::ADDRESS,
    device_access_status_len = DeviceAccessStatus::LENGTH,
    device_access_status_access = DeviceAccessStatus::ACCESS_RIGHT.as_str(),
    stream_selector_addr = StreamSelector::ADDRESS,
    stream_selector_len = StreamSelector::LENGTH,
    stream_selector_access = StreamSelector::ACCESS_RIGHT.as_str(),
    stream_selector_max_addr = StreamSelectorMax::ADDRESS,
    stream_selector_max_len = StreamSelectorMax::LENGTH,
    stream_selector_max_access = StreamSelectorMax::ACCESS_RIGHT.as_str(),
    stream_id_addr = StreamID::ADDRESS,
    stream_id_len = StreamID::LENGTH,
    stream_id_access = StreamID::ACCESS_RIGHT.as_str(),
);
