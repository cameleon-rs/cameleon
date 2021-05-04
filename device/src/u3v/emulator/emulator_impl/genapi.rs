use cameleon_impl::memory::{prelude::*, register_map};
use const_format::formatcp;

use super::memory::GENAPI_XML_ADDRESS;

pub(super) const MODEL_NAME: &str = "CameleonU3VEmulator";
pub(super) const VENDOR_NAME: &str = "CameleonProjectDevelopers";

pub(super) const XML_MAJOR_VERSION: u64 = 1;
pub(super) const XML_MINOR_VERSION: u64 = 0;
pub(super) const XML_SUBMINOR_VERSION: u64 = 0;

pub(super) const SCHEME_MAJOR_VERSION: u64 = 1;
pub(super) const SCHEME_MINOR_VERSION: u64 = 1;
pub(super) const SCHEME_SUBMINOR_VERSION: u64 = 0;

pub(super) const TOOL_TIP: &str = "CameleonU3VEmulator";

pub(super) const PORT_NAME: &str = "Device";

const PRODUCT_GUID: &str = "eaabe337-2c3b-4e0b-b9b9-e67b347c4da8";
const VERSION_GUID: &str = "0d29949b-5cd9-4f08-93fb-eea24950de3f";

#[register_map(base=GENAPI_XML_ADDRESS, endianness=LE)]
pub(super) enum GenApiReg {
    /// Start acquisition of images when the register is set to 1.
    #[register(len = 1, access = WO, ty = u8)]
    AcquisitionStart,

    /// Stop the acquisition of images when the register is set to 1.
    #[register(len = 1, access = WO, ty = u8)]
    AcquisitionStop,
}

// TODO: Add node to this XML.
pub(super) const GENAPI_XML: &str = formatcp!(
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
        <pFeature>AcquisitionControl</pFeature>
    </Category>

    <Port Name="{PORT_NAME}" NameSpace="Standard">
        <Description>The GenICam port through which the Interface module is accessed.</Description>
        <Visibility>Invisible</Visibility>
    </Port>

    <Category Name="AcquisitionControl" NameSpace="Standard">
        <DisplayName>Acquisition Control</DisplayName>
        <pFeature>AcquisitionStart</pFeature>
        <pFeature>AcquisitionStop</pFeature>
    </Category>

    <Command Name="AcquisitionStart" NameSpace="Standard">
        <ToolTip>Starts the acquisition of images.</ToolTip>
        <Description>This command starts the acquisition of images.</Description>
        <DisplayName>Acquisition Start</DisplayName>
        <pValue>AcquisitionStartReg</pValue>
        <CommandValue>1</CommandValue>
    </Command>

    <IntReg Name="AcquisitionStartReg" NameSpace="Custom">
        <Address>{acquisition_start_addr}</Address>
        <Length>{acquisition_start_len}</Length>
        <AccessMode>{acquisition_start_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Sign>Unsigned</Sign>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

   <Command Name="AcquisitionStop" NameSpace="Standard">
        <ToolTip>Stops the acquisition of images.</ToolTip>
        <Description>This command stop the acquisition of images.</Description>
        <DisplayName>Acquisition Stop</DisplayName>
        <pValue>AcquisitionStopReg</pValue>
        <CommandValue>1</CommandValue>
    </Command>

    <IntReg Name="AcquisitionStopReg" NameSpace="Custom">
        <Address>{acquisition_stop_addr}</Address>
        <Length>{acquisition_stop_len}</Length>
        <AccessMode>{acquisition_stop_access}</AccessMode>
        <pPort>{PORT_NAME}</pPort>
        <Sign>Unsigned</Sign>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

</RegisterDescription>"#,
    acquisition_start_addr = GenApiReg::AcquisitionStart::ADDRESS,
    acquisition_start_len = GenApiReg::AcquisitionStart::LENGTH,
    acquisition_start_access = GenApiReg::AcquisitionStart::ACCESS_RIGHT.as_str(),
    acquisition_stop_addr = GenApiReg::AcquisitionStop::ADDRESS,
    acquisition_stop_len = GenApiReg::AcquisitionStop::LENGTH,
    acquisition_stop_access = GenApiReg::AcquisitionStop::ACCESS_RIGHT.as_str(),
);
