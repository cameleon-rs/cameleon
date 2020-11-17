use cameleon_impl::memory::{genapi, memory};

use crate::imp::port::TlType;

#[memory]
pub(super) struct Memory {
    genapi: GenApi,
}

#[genapi(endianness = LE)]
pub(super) enum GenApi {
    XML = r#"<RegisterDescription
    ModelName="CameleonGenTLSystemModule"
    VendorName="CameleonProjectDevelopers"
    StandardNameSpace="None"
    SchemaMajorVersion="1"
    SchemaMinorVersion="1"
    SchemaSubMinorVersion="0"
    MajorVersion="1"
    MinorVersion="1"
    SubMinorVersion="0"
    ToolTip="GenTL System Module"
    ProductGuid="C09F0257-3F5C-41C2-B34F-FE67CB108370"
    VersionGuid="10F7AF60-A1B0-4AE4-8785-F214C22DAA9D"
    xmlns="http://www.genicam.org/GenApi/Version_1_1"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.genicam.org/GenApi/Version_1_1 http://www.genicam.org/GenApi/GenApiSchema_Version_1_1.xsd">

    <Category Name="Root" NameSpace="Standard">
        <Description>Provides the Root of the GenICam features tree.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>SystemInformation</pFeature>
        <pFeature>InterfaceEnumeration</pFeature>
    </Category>

    <Port Name="TLPort" NameSpace="Standard">
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

        <Value>C09F0257-3F5C-41C2-B34F-FE67CB108370</Value>
    </String>

    <String Name="TLVendorName" NameSpace="Standard">
        <Description>Name of the GenTL Producer vendor.</Description>
        <Visibility>Beginner</Visibility>

        <Value>Cameleon Project Developers</Value>
    </String>

    <String Name="TLModelName" NameSpace="Standard">
        <Description>Name of the GenTL Producer to distinguish different kinds of GenTL Producer implementations from one vendor.</Description>
        <Visibility>Beginner</Visibility>

        <Value>Cameleon GenTL System Module</Value>
    </String>

    <String Name="TLVersion" NameSpace="Standard">
        <Description>Vendor specific version string of the GenTL Producer.</Description>
        <Visibility>Beginner</Visibility>

        <Value>1.1.0</Value>
    </String>

    <String Name="TLPath" NameSpace="Standard">
        <Description>Vendor specific version string of the GenTL Producer.</Description>
        <Visibility>Beginner</Visibility>

        <pValue>TLPathReg</pValue>
    </String>

    <Enumeration Name="TLType" NameSpace="Standard">
        <Description>Transport layer type of the GenTL Producer implementation.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="Mixed" NameSpace="Standard">
            <Description>Different Interface modules of the GenTL Producer are of different types.</Description>
            <Value>0</Value>
        </EnumEntry>
        <Value>0</Value>
    </Enumeration>

    <Integer Name="GenTLVersionMajor" NameSpace="Standard">
        <Description>Major version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>1</Value>
        <Min>1</Min>
        <Max>1</Max>
    </Integer>

    <Integer Name="GenTLVersionMinor" NameSpace="Standard">
        <Description>Minor version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>6</Value>
        <Min>6</Min>
        <Max>6</Max>
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

    <Integer Name="InterfaceSelector" NameSpace="Standard">
        <Description>Selector for the different GenTL Producer interfaces.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>InterfaceSelectorReg</pValue>
        <Min>0</Min>
        <pMax>InterfaceSelectorMaxReg</pMax>
    </Integer>

    <String Name="InterfaceID" NameSpace="Standard">
        <Description>GenTL Producer wide unique identifier of the selected interface.</Description>
        <Visibility>Beginner</Visibility>
        <pValue>InterfaceIDReg</pValue>
    </String>

    <Integer Name="GevInterfaceMACAddress" NameSpace="Standard">
        <Description>48-bit MAC address of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <pValue>GevInterfaceMACAddressReg</pValue>
        <Representation>MACAddress</Representation>
    </Integer>

    <Integer Name="GevInterfaceDefaultIPAddress" NameSpace="Standard">
        <Description>IP address of the first subnet of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <pValue>GevInterfaceDefaultIPAddressReg</pValue>
        <Representation>IPV4Address</Representation>
    </Integer>

    <Integer Name="GevInterfaceDefaultSubnetMask" NameSpace="Standard">
        <Description>Subnet mask of the first subnet of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <pValue>GevInterfaceDefaultSubnetMaskReg</pValue>
        <Representation>IPV4Address</Representation>
    </Integer>

    <Integer Name="GevInterfaceDefaultGateway" NameSpace="Standard">
        <Description>Gateway of the selected interface.</Description>
        <Visibility>Expert</Visibility>
        <pValue>GevInterfaceDefaultGatewayReg</pValue>
        <Representation>IPV4Address</Representation>
    </Integer>

    <!-- Implementation details start -->

    <StringReg Name="TLPathReg" NameSpace="Standard">
        <Visibility>Invisible</Visibility>
        <Address>0</Address>
        <Length>1024</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
    </StringReg>

    <IntReg Name="InterfaceUpdateListReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1024</Address>
        <Length>4</Length>
        <AccessMode>WO</AccessMode>
        <pPort>TLPort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="InterfaceSelectorReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1028</Address>
        <Length>4</Length>
        <AccessMode>RW</AccessMode>
        <pPort>TLPort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="InterfaceSelectorMaxReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1032</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <StringReg Name="InterfaceIDReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1036</Address>
        <Length>64</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
    </StringReg>

    <MaskedIntReg Name="GevInterfaceMACAddressReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1100</Address>
        <Length>8</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
        <LSB>0</LSB>
        <MSB>47</MSB>
        <Endianess>LittleEndian</Endianess>
    </MaskedIntReg>

    <IntReg Name="GevInterfaceDefaultIPAddressReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1108</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="GevInterfaceDefaultSubnetMaskReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1112</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="GevInterfaceDefaultGatewayReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>1116</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>TLPort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

</RegisterDescription>
"#,
}

impl Into<TlType> for GenApi::TLType {
    fn into(self) -> TlType {
        match self {
            GenApi::TLType::Mixed => TlType::Mixed,
        }
    }
}
