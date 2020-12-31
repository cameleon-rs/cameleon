use cameleon_impl::memory::{genapi, memory};

use crate::imp::port::TlType;

use super::DeviceAccessStatus;

#[memory]
pub(super) struct Memory {
    genapi: GenApi,
}

#[genapi(endianness = LE)]
pub(super) enum GenApi {
    XML = r#"<RegisterDescription
    ModelName="CameleonGenTLU3VDeviceModule"
    VendorName="CameleonProjectDevelopers"
    StandardNameSpace="None"
    SchemaMajorVersion="1"
    SchemaMinorVersion="1"
    SchemaSubMinorVersion="0"
    MajorVersion="1"
    MinorVersion="1"
    SubMinorVersion="0"
    ToolTip="GenTL U3V Device Module"
    ProductGuid="21fbd4d2-6244-445b-827f-6fd92a8787c8"
    VersionGuid="0cd40ca9-9db8-49db-9952-cb758555d04e"
    xmlns="http://www.genicam.org/GenApi/Version_1_1"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.genicam.org/GenApi/Version_1_1 http://www.genicam.org/GenApi/GenApiSchema_Version_1_1.xsd">

    <Category Name="Root" NameSpace="Standard">
        <Description>Provides the Root of the GenICam features tree.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>DeviceInformation</pFeature>
        <pFeature>StreamEnumeration</pFeature>
    </Category>

    <Port Name="DevicePort" NameSpace="Standard">
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

    <String Name="DeviceID" NameSpace="Standard">
        <Description>Interface wide unique identifier of the selected device.</Description>
        <Visibility>Expert</Visibility>
        <pValue>DeviceIDReg</pValue>
    </String>

    <String Name="DeviceVendorName" NameSpace="Standard">
        <Description>Name of the device vendor.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>DeviceVendorNameReg</pValue>
    </String>

    <String Name="DeviceModelName" NameSpace="Standard">
        <Description>Name of the device model.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>DeviceModelNameReg</pValue>
    </String>

    <Enumeration Name="DeviceType" NameSpace="Standard">
        <Description>Transport layer type of the device.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="USB3Vision" NameSpace="Standard">
            <Description>USB3 Vision</Description>
            <Value>0</Value>
        </EnumEntry>
        <Value>0</Value>
    </Enumeration>

    <Enumeration Name="DeviceAccessStatus" NameSpace="Standard">
        <Description>Gives the device's access status at the moment of the last execution of the DeviceUpdateList command.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="Unknown" NameSpace="Standard">
            <Description>Not known to producer.</Description>
            <Value>0</Value>
        </EnumEntry>

        <EnumEntry Name="ReadWrite" NameSpace="Standard">
            <Description>Full access.</Description>
            <Value>1</Value>
        </EnumEntry>

        <EnumEntry Name="ReadOnly" NameSpace="Standard">
            <Description>Read-only access.</Description>
            <Value>2</Value>
        </EnumEntry>

        <EnumEntry Name="NoAccess" NameSpace="Standard">
            <Description>Not available to connect.</Description>
            <Value>3</Value>
        </EnumEntry>

        <EnumEntry Name="Busy" NameSpace="Standard">
            <Description>The device is already opened by another entity.</Description>
            <Value>4</Value>
        </EnumEntry>

        <EnumEntry Name="OpenReadWrite" NameSpace="Standard">
            <Description>Open in Read/Write mode by this GenTL host.</Description>
            <Value>5</Value>
        </EnumEntry>

        <EnumEntry Name="OpenReadOnly" NameSpace="Standard">
            <Description>Open in Read only mode by this GenTL host</Description>
            <Value>6</Value>
        </EnumEntry>

        <pValue>DeviceAccessStatusReg</pValue>
    </Enumeration>

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

    <String Name="StreamID" NameSpace="Standard">
        <Description>Device unique ID for the stream.</Description>
        <Visibility>Expert</Visibility>
        <pValue>StreamIDReg</pValue>
    </String>


    <!-- Implementation details start -->

    <StringReg Name="DeviceIDReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>0</Address>
        <Length>64</Length>
        <AccessMode>RO</AccessMode>
        <pPort>DevicePort</pPort>
    </StringReg>

    <StringReg Name="DeviceVendorNameReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>64</Address>
        <Length>128</Length>
        <AccessMode>RO</AccessMode>
        <pPort>DevicePort</pPort>
    </StringReg>

    <StringReg Name="DeviceModelNameReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>192</Address>
        <Length>128</Length>
        <AccessMode>RO</AccessMode>
        <pPort>DevicePort</pPort>
    </StringReg>

    <IntReg Name="DeviceAccessStatusReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>320</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>DevicePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="StreamSelectorReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>324</Address>
        <Length>4</Length>
        <AccessMode>RW</AccessMode>
        <pPort>DevicePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="StreamSelectorMaxReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>328</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>DevicePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <StringReg Name="StreamIDReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>332</Address>
        <Length>64</Length>
        <AccessMode>RO</AccessMode>
        <pPort>DevicePort</pPort>
    </StringReg>

</RegisterDescription>
"#,
}

impl Into<TlType> for GenApi::DeviceType {
    fn into(self) -> TlType {
        match self {
            GenApi::DeviceType::USB3Vision => TlType::USB3Vision,
        }
    }
}

impl Into<DeviceAccessStatus> for GenApi::DeviceAccessStatus {
    fn into(self) -> DeviceAccessStatus {
        use DeviceAccessStatus::*;

        match self {
            GenApi::DeviceAccessStatus::Unknown => Unknown,
            GenApi::DeviceAccessStatus::ReadWrite => ReadWrite,
            GenApi::DeviceAccessStatus::ReadOnly => ReadOnly,
            GenApi::DeviceAccessStatus::NoAccess => NoAccess,
            GenApi::DeviceAccessStatus::Busy => Busy,
            GenApi::DeviceAccessStatus::OpenReadWrite => OpenReadWrite,
            GenApi::DeviceAccessStatus::OpenReadOnly => OpenReadOnly,
        }
    }
}

impl From<DeviceAccessStatus> for GenApi::DeviceAccessStatus {
    fn from(status: DeviceAccessStatus) -> Self {
        use DeviceAccessStatus::*;

        match status {
            Unknown => GenApi::DeviceAccessStatus::Unknown,
            ReadWrite => GenApi::DeviceAccessStatus::ReadWrite,
            ReadOnly => GenApi::DeviceAccessStatus::ReadOnly,
            NoAccess => GenApi::DeviceAccessStatus::NoAccess,
            Busy => GenApi::DeviceAccessStatus::Busy,
            OpenReadWrite => GenApi::DeviceAccessStatus::OpenReadWrite,
            OpenReadOnly => GenApi::DeviceAccessStatus::OpenReadOnly,
        }
    }
}

impl GenApi::DeviceAccessStatus {
    pub(crate) fn from_num(num: isize) -> Self {
        use GenApi::DeviceAccessStatus::*;

        match num {
            _ if num == Unknown as isize => Unknown,
            _ if num == ReadWrite as isize => ReadWrite,
            _ if num == ReadOnly as isize => ReadOnly,
            _ if num == NoAccess as isize => NoAccess,
            _ if num == Busy as isize => Busy,
            _ if num == OpenReadWrite as isize => OpenReadWrite,
            _ if num == OpenReadOnly as isize => OpenReadOnly,
            _ => unreachable!(),
        }
    }
}
