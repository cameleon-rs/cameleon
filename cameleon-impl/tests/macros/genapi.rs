use cameleon_impl::memory::{genapi, *};

#[genapi(xml_base = 0x10000, endianness = LE)]
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


 </RegisterDescription>
"#,
}

fn main() {
    let raw_reg = GenApi::MyIntReg::raw();
    assert_eq!(raw_reg.offset, 20000);
    assert_eq!(raw_reg.len, 8);

    let raw_reg = GenApi::MyMaskedIntReg::raw();
    assert_eq!(raw_reg.offset, 20008);
    assert_eq!(raw_reg.len, 4);
}
