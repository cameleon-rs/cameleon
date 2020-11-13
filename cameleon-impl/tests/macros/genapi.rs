use cameleon_impl::memory::{genapi, memory, prelude::*, Register};

#[memory]
pub struct Memory {
    gen_api: GenApi,
}

#[genapi(xml_base = 10000, endianness = LE)]
pub enum GenApi {
    XML = r#"
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
        <pFeature>MyIntReg</pFeature>
        <pFeature>MyMaskedIntReg</pFeature>
        <pFeature>MyFloatReg</pFeature>
        <pFeature>MyStringReg</pFeature>
        <pFeature>MyRegister</pFeature>
        <pFeature>MyStructEntry1</pFeature>
        <pFeature>MyStructEntry2</pFeature>
    </Category>

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


</RegisterDescription>
"#,
}

fn main() {
    let raw_reg = GenApi::XML::raw();
    assert_eq!(raw_reg.offset, 10000);

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

    assert_eq!(
        memory.read::<GenApi::XML>().unwrap(),
        r#"
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
        <pFeature>MyIntReg</pFeature>
        <pFeature>MyMaskedIntReg</pFeature>
        <pFeature>MyFloatReg</pFeature>
        <pFeature>MyStringReg</pFeature>
        <pFeature>MyRegister</pFeature>
        <pFeature>MyStructEntry1</pFeature>
        <pFeature>MyStructEntry2</pFeature>
    </Category>

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


</RegisterDescription>
"#
        .as_bytes()
        .to_owned()
    );
}