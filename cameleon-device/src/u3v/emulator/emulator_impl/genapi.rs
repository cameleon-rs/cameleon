use const_format::formatcp;

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
        <pFeature>DeviceEnumeration</pFeature>
    </Category>

    <Port Name="{PORT_NAME}" NameSpace="Standard">
        <Description>The GenICam port through which the Interface module is accessed.</Description>
        <Visibility>Invisible</Visibility>
    </Port>
</RegisterDescription>"#,
);
