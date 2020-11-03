use cameleon_impl::memory::{genapi, memory};

#[memory]
pub(super) struct Memory {
    genapi: GenApi,
}

#[genapi(endianness = LE)]
pub(super) enum GenApi {
    XML = r#"<RegisterDescription
    ModelName="CameleonGenTLU3VInterfaceModule"
    VendorName="CameleonProjectDevelopers"
    StandardNameSpace="None"
    SchemaMajorVersion="1"
    SchemaMinorVersion="1"
    SchemaSubMinorVersion="0"
    MajorVersion="1"
    MinorVersion="1"
    SubMinorVersion="0"
    ToolTip="GenTL U3V Interface Module"
    ProductGuid="639290f8-043c-436d-b8d1-cb916e2928e9"
    VersionGuid="fee92fd7-f388-44c7-bd07-c7f706bdc182"
    xmlns="http://www.genicam.org/GenApi/Version_1_1"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.genicam.org/GenApi/Version_1_1 http://www.genicam.org/GenApi/GenApiSchema_Version_1_1.xsd">

    <Category Name="Root" NameSpace="Standard">
        <Description>Provides the Root of the GenICam features tree.</Description>
        <Visibility>Beginner</Visibility>

        <pFeature>InterfaceInformation</pFeature>
        <pFeature>DeviceEnumeration</pFeature>
    </Category>

    <Port Name="InterfacePort" NameSpace="Standard">
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

        <pValue>InterfaceIDReg</pValue>
    </String>

    <Enumeration Name="InterfaceType" NameSpace="Standard">
        <Description>Transport layer type of the interface.</Description>
        <Visibility>Expert</Visibility>
        <EnumEntry Name="USB3Vision" NameSpace="Standard">
            <Description>USB3 Vision</Description>
            <Value>0</Value>
        </EnumEntry>
        <Value>0</Value>
    </Enumeration>

    <Integer Name="InterfaceTLVersionMajor" NameSpace="Standard">
        <Description>Major version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>1</Value>
        <Min>1</Min>
        <Max>1</Max>
    </Integer>

    <Integer Name="InterfaceTLVersionMinor" NameSpace="Standard">
        <Description>Minor version number of the GenTL specification the GenTL Producer implementation complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>6</Value>
        <Min>6</Min>
        <Max>6</Max>
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

    <Integer Name="DeviceSelector" NameSpace="Standard">
        <Description>Selector for the different devices on this interface.</Description>
        <Visibility>Expert</Visibility>
        <pValue>DeviceSelectorReg</pValue>
        <Min>0</Min>
        <pMax>DeviceSelectorMaxReg</pMax>
    </Integer>

    <String Name="DeviceID" NameSpace="Standard">
        <Description>Interface wide unique identifier of the selected device.</Description>
        <Visibility>Expert</Visibility>
        <pValue>DeviceIDReg</pValue>
    </String>

    <String Name="DeviceVendorName" NameSpace="Standard">
        <Description>Name of the device vendor.</Description>
        <Visibility>Expert</Visibility>
        <pValue>DeviceVendorNameReg</pValue>
    </String>

    <String Name="DeviceModelName" NameSpace="Standard">
        <Description>Name of the device model.</Description>
        <Visibility>Expert</Visibility>
        <pValue>DeviceModelNameReg</pValue>
    </String>

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

    <Integer Name="DeviceTLVersionMajor" NameSpace="Standard">
        <Description>Major version number of the transport layer specification the remote device complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>1</Value>
        <Min>1</Min>
        <Max>1</Max>
    </Integer>

    <Integer Name="DeviceTLVersionMinor" NameSpace="Standard">
        <Description>Minor version number of the transport layer specification the remote device complies with.</Description>
        <Visibility>Expert</Visibility>
        <Value>6</Value>
        <Min>6</Min>
        <Max>6</Max>
    </Integer>

    <!-- Implementation details start -->

    <StringReg Name="InterfaceIDReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>0</Address>
        <Length>64</Length>
        <AccessMode>RO</AccessMode>
        <pPort>InterfacePort</pPort>
    </StringReg>

    <IntReg Name="DeviceUpdateListReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>64</Address>
        <Length>4</Length>
        <AccessMode>WO</AccessMode>
        <pPort>InterfacePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="DeviceSelectorReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>68</Address>
        <Length>4</Length>
        <AccessMode>RW</AccessMode>
        <pPort>InterfacePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <IntReg Name="DeviceSelectorMaxReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>72</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>InterfacePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

    <StringReg Name="DeviceIDReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>76</Address>
        <Length>64</Length>
        <AccessMode>RO</AccessMode>
        <pPort>InterfacePort</pPort>
    </StringReg>

    <StringReg Name="DeviceVendorNameReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>140</Address>
        <Length>128</Length>
        <AccessMode>RO</AccessMode>
        <pPort>InterfacePort</pPort>
    </StringReg>

    <StringReg Name="DeviceModelNameReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>268</Address>
        <Length>128</Length>
        <AccessMode>RO</AccessMode>
        <pPort>InterfacePort</pPort>
    </StringReg>

    <IntReg Name="DeviceAccessStatusReg" NameSpace="Custom">
        <Visibility>Invisible</Visibility>
        <Address>396</Address>
        <Length>4</Length>
        <AccessMode>RO</AccessMode>
        <pPort>InterfacePort</pPort>
        <Endianess>LittleEndian</Endianess>
    </IntReg>

</RegisterDescription>
"#,
}
