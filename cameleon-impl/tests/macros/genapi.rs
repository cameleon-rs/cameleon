use cameleon_impl::memory::{genapi, memory, prelude::*, Register};

#[memory]
pub struct Memory {
    gen_api: GenApi,
}

#[genapi(endianness = LE)]
pub enum GenApi {
    Xml = r#"
<RegisterDescription
    ModelName="CameleonModel"
    VendorName="CameleonVendor"
    StandardNameSpace="None"
    SchemaMajorVersion="1"
    SchemaMinorVersion="1"
    SchemaSubMinorVersion="0"
    MajorVersion="1"
    MinorVersion="2"
    SubMinorVersion="3"
    ToolTip="ToolTiptest"
    ProductGuid="01234567-0123-0123-0123-0123456789ab"
    VersionGuid="76543210-3210-3210-3210-ba9876543210"
    xmlns="http://www.genicam.org/GenApi/Version_1_0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.genicam.org/GenApi/Version_1_0 GenApiSchema.xsd">

    <Category Name="Root" NameSpace="Standard">
        <pFeature>MyInteger</pFeature>
        <pFeature>MyIntReg</pFeature>
        <pFeature>MyMaskedIntReg</pFeature>
        <pFeature>MyFloatReg</pFeature>
        <pFeature>MyStringReg</pFeature>
        <pFeature>MyRegister</pFeature>
        <pFeature>MyStructEntry1</pFeature>
        <pFeature>MyStructEntry2</pFeature>
    </Category>

    <Integer Name="MyInteger">
      <Value>10</Value>
    </Integer>

    <Port Name="Device" NameSpace="Standard">
    </Port>

    <IntReg Name="MyIntReg">
      <Address>20000</Address>
      <Length>8</Length>
      <pPort>Device</pPort>
    </IntReg>

    <MaskedIntReg Name="MyMaskedIntReg">
      <Address>20008</Address>
      <Length>4</Length>
      <pPort>Device</pPort>
      <LSB>3</LSB>
      <MSB>7</MSB>
    </MaskedIntReg>

    <FloatReg Name="MyFloatReg">
      <Address>1000000</Address>
      <Length>4</Length>
      <pPort>Device</pPort>
    </FloatReg>

    <StringReg Name="MyStringReg">
      <Address>20016</Address>
      <Length>128</Length>
      <pPort>Device</pPort>
    </StringReg>

    <Register Name="MyRegister">
      <Address>21000</Address>
      <Length>64</Length>
      <pPort>Device</pPort>
    </Register>

    <StructReg Comment="Struct Entry Comment">
        <Address>30000</Address>
        <Length>4</Length>
        <pPort>Device</pPort>
        <Endianess>BigEndian</Endianess>

        <StructEntry Name="MyStructEntry1">
            <Bit>24</Bit>
        </StructEntry>

        <StructEntry Name="MyStructEntry2">
            <LSB>4</LSB>
            <MSB>4</MSB>
        </StructEntry>
    </StructReg>

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

        <Value>3</Value>
    </Enumeration>

</RegisterDescription>
"#,
}

fn main() {
    let xml_str = r#"
<RegisterDescription
    ModelName="CameleonModel"
    VendorName="CameleonVendor"
    StandardNameSpace="None"
    SchemaMajorVersion="1"
    SchemaMinorVersion="1"
    SchemaSubMinorVersion="0"
    MajorVersion="1"
    MinorVersion="2"
    SubMinorVersion="3"
    ToolTip="ToolTiptest"
    ProductGuid="01234567-0123-0123-0123-0123456789ab"
    VersionGuid="76543210-3210-3210-3210-ba9876543210"
    xmlns="http://www.genicam.org/GenApi/Version_1_0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://www.genicam.org/GenApi/Version_1_0 GenApiSchema.xsd">

    <Category Name="Root" NameSpace="Standard">
        <pFeature>MyInteger</pFeature>
        <pFeature>MyIntReg</pFeature>
        <pFeature>MyMaskedIntReg</pFeature>
        <pFeature>MyFloatReg</pFeature>
        <pFeature>MyStringReg</pFeature>
        <pFeature>MyRegister</pFeature>
        <pFeature>MyStructEntry1</pFeature>
        <pFeature>MyStructEntry2</pFeature>
    </Category>

    <Integer Name="MyInteger">
      <Value>10</Value>
    </Integer>

    <Port Name="Device" NameSpace="Standard">
    </Port>

    <IntReg Name="MyIntReg">
      <Address>20000</Address>
      <Length>8</Length>
      <pPort>Device</pPort>
    </IntReg>

    <MaskedIntReg Name="MyMaskedIntReg">
      <Address>20008</Address>
      <Length>4</Length>
      <pPort>Device</pPort>
      <LSB>3</LSB>
      <MSB>7</MSB>
    </MaskedIntReg>

    <FloatReg Name="MyFloatReg">
      <Address>1000000</Address>
      <Length>4</Length>
      <pPort>Device</pPort>
    </FloatReg>

    <StringReg Name="MyStringReg">
      <Address>20016</Address>
      <Length>128</Length>
      <pPort>Device</pPort>
    </StringReg>

    <Register Name="MyRegister">
      <Address>21000</Address>
      <Length>64</Length>
      <pPort>Device</pPort>
    </Register>

    <StructReg Comment="Struct Entry Comment">
        <Address>30000</Address>
        <Length>4</Length>
        <pPort>Device</pPort>
        <Endianess>BigEndian</Endianess>

        <StructEntry Name="MyStructEntry1">
            <Bit>24</Bit>
        </StructEntry>

        <StructEntry Name="MyStructEntry2">
            <LSB>4</LSB>
            <MSB>4</MSB>
        </StructEntry>
    </StructReg>

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

        <Value>3</Value>
    </Enumeration>

</RegisterDescription>
"#
    .as_bytes()
    .to_owned();

    let xml_address = GenApi::xml_address();
    assert_eq!(xml_address, 1000004);

    let xml_length = GenApi::xml_length();
    assert_eq!(xml_length, xml_str.len());

    let genapi_version = GenApi::genapi_version();
    assert_eq!(genapi_version, semver::Version::new(1, 2, 3));

    let schema_version = GenApi::schema_version();
    assert_eq!(schema_version, semver::Version::new(1, 1, 0));

    let vendor_name = GenApi::vendor_name();
    assert_eq!(vendor_name, "CameleonVendor");

    assert_eq!(GenApi::MyInteger, 10);

    assert_eq!(GenApi::DeviceAccessStatus::Unknown as isize, 0);
    assert_eq!(GenApi::DeviceAccessStatus::ReadWrite as isize, 1);
    assert_eq!(GenApi::DeviceAccessStatus::OpenReadOnly as isize, 6);

    assert_eq!(GenApi::Device, "Device");

    let raw_reg = GenApi::MyIntReg::raw();
    assert_eq!(raw_reg.offset, 20000);
    assert_eq!(raw_reg.len, 8);

    let raw_reg = GenApi::MyMaskedIntReg::raw();
    assert_eq!(raw_reg.offset, 20008);
    assert_eq!(raw_reg.len, 4);

    let raw_reg = GenApi::MyFloatReg::raw();
    assert_eq!(raw_reg.offset, 1000000);
    assert_eq!(raw_reg.len, 4);

    let raw_reg = GenApi::MyStringReg::raw();
    assert_eq!(raw_reg.offset, 20016);
    assert_eq!(raw_reg.len, 128);

    let raw_reg = GenApi::MyRegister::raw();
    assert_eq!(raw_reg.offset, 21000);
    assert_eq!(raw_reg.len, 64);

    let raw_reg = GenApi::MyStructEntry1::raw();
    assert_eq!(raw_reg.offset, 30000);
    assert_eq!(raw_reg.len, 4);

    let raw_reg = GenApi::MyStructEntry2::raw();
    assert_eq!(raw_reg.offset, 30000);
    assert_eq!(raw_reg.len, 4);

    let memory = Memory::new();

    assert_eq!(&memory.read::<GenApi::Xml>().unwrap(), &xml_str);
}
