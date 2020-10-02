use super::{elem_name::*, elem_type::StandardNameSpace, *};

pub struct RegisterDescription {
    model_name: String,
    vendor_name: String,
    tool_tip: Option<String>,
    standard_name_space: StandardNameSpace,
    schema_major_version: u64,
    schema_minor_version: u64,
    schema_subminor_version: u64,
    major_version: u64,
    minor_version: u64,
    subminor_version: u64,
    product_guid: String,
    version_guid: String,

    nodes: Vec<NodeKind>,
}

impl RegisterDescription {
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn vendor_name(&self) -> &str {
        &self.vendor_name
    }

    pub fn tool_tip(&self) -> Option<&str> {
        self.tool_tip.as_deref()
    }

    pub fn standard_name_space(&self) -> StandardNameSpace {
        self.standard_name_space
    }

    pub fn schema_major_version(&self) -> u64 {
        self.schema_major_version
    }

    pub fn schema_subminor_version(&self) -> u64 {
        self.schema_subminor_version
    }

    pub fn schema_minor_version(&self) -> u64 {
        self.schema_minor_version
    }

    pub fn major_version(&self) -> u64 {
        self.major_version
    }

    pub fn minor_version(&self) -> u64 {
        self.minor_version
    }

    pub fn subminor_version(&self) -> u64 {
        self.subminor_version
    }

    pub fn product_guid(&self) -> &str {
        &self.product_guid
    }

    pub fn version_guid(&self) -> &str {
        &self.version_guid
    }

    pub fn nodes(&self) -> &[NodeKind] {
        &self.nodes
    }
}

impl Parse for RegisterDescription {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), REGISTER_DESCRIPTION);

        let model_name = node.attribute_of(MODEL_NAME).unwrap().into();
        let vendor_name = node.attribute_of(VENDOR_NAME).unwrap().into();
        let tool_tip = node.attribute_of(TOOL_TIP).map(Into::into);
        let standard_name_space = node.attribute_of(STANDARD_NAME_SPCACE).unwrap().into();
        let schema_major_version =
            convert_to_uint(&node.attribute_of(SCHEMA_MAJOR_VERSION).unwrap());
        let schema_minor_version =
            convert_to_uint(&node.attribute_of(SCHEMA_MINOR_VERSION).unwrap());
        let schema_subminor_version =
            convert_to_uint(&node.attribute_of(SCHEMA_SUB_MINOR_VERSION).unwrap());
        let major_version = convert_to_uint(&node.attribute_of(MAJOR_VERSION).unwrap());
        let minor_version = convert_to_uint(&node.attribute_of(MINOR_VERSION).unwrap());
        let subminor_version = convert_to_uint(&node.attribute_of(SUB_MINOR_VERSION).unwrap());
        let product_guid = node.attribute_of(PRODUCT_GUID).unwrap().into();
        let version_guid = node.attribute_of(VERSION_GUID).unwrap().into();

        let mut nodes = vec![];
        while let Some(ref mut child) = node.next() {
            let node = match child.tag_name() {
                NODE => NodeKind::Node(Box::new(child.parse())),
                CATEGORY => NodeKind::Category(Box::new(child.parse())),
                INTEGER => NodeKind::Integer(Box::new(child.parse())),
                INT_REG => NodeKind::IntReg(Box::new(child.parse())),
                MASKED_INT_REG => NodeKind::MaskedIntReg(Box::new(child.parse())),
                BOOLEAN => NodeKind::Boolean(Box::new(child.parse())),
                COMMAND => NodeKind::Command(Box::new(child.parse())),
                ENUMERATION => NodeKind::Enumeration(Box::new(child.parse())),
                FLOAT => NodeKind::Float(Box::new(child.parse())),
                FLOAT_REG => NodeKind::FloatReg(Box::new(child.parse())),
                STRING_REG => NodeKind::StringReg(Box::new(child.parse())),
                REGISTER => NodeKind::Register(Box::new(child.parse())),
                SWISS_KNIFE => NodeKind::SwissKnife(Box::new(child.parse())),
                INT_SWISS_KNIFE => NodeKind::IntSwissKnife(Box::new(child.parse())),
                STRUCT_REG => NodeKind::StructReg(Box::new(child.parse())),
                _ => todo!(),
            };
            nodes.push(node);
        }

        Self {
            model_name,
            vendor_name,
            tool_tip,
            standard_name_space,
            schema_major_version,
            schema_minor_version,
            schema_subminor_version,
            major_version,
            minor_version,
            subminor_version,
            product_guid,
            version_guid,
            nodes,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Node(Box<Node>),
    Category(Box<CategoryNode>),
    Integer(Box<IntegerNode>),
    IntReg(Box<IntRegNode>),
    MaskedIntReg(Box<MaskedIntRegNode>),
    Boolean(Box<BooleanNode>),
    Command(Box<CommandNode>),
    Enumeration(Box<EnumerationNode>),
    Float(Box<FloatNode>),
    FloatReg(Box<FloatRegNode>),
    StringReg(Box<StringRegNode>),
    Register(Box<RegisterNode>),
    SwissKnife(Box<SwissKnifeNode>),
    IntSwissKnife(Box<IntSwissKnifeNode>),
    StructReg(Box<StructRegNode>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_description() {
        let xml = r#"
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
                <pFeature>MyNode</pFeature>
                <pFeature>MyInt</pFeature>
                <pFeature>MyIntReg</pFeature>
                <pFeature>MyMaskedIntReg</pFeature>
                <pFeature>MyBoolean</pFeature>
                <pFeature>MyCommand</pFeature>
                <pFeature>MyEnumeration</pFeature>
                <pFeature>MyFloat</pFeature>
                <pFeature>MyFloatReg</pFeature>
                <pFeature>MyStringReg</pFeature>
                <pFeature>MyRegister</pFeature>
                <pFeature>MySwissKnife</pFeature>
                <pFeature>MyIntSwissKnife</pFeature>
                <pFeature>MyStructEntry</pFeature>
            </Category>

            <Node Name = "MyNode"></Node>

            <Integer Name="MyInt">
                <Value>10</Value>
            </Integer>

            <IntReg Name="MyIntReg">
              <Address>0x10000</Address>
              <pLength>LengthNode</pLength>
              <pPort>Device</pPort>
            </IntReg>

            <MaskedIntReg Name="MyMaskedIntReg">
              <Address>0x10000</Address>
              <Length>4</Length>
              <pPort>Device</pPort>
              <LSB>3</LSB>
              <MSB>7</MSB>
            </MaskedIntReg>

            <Boolean Name="MyBoolean">
                <pValue>Node</pValue>
                <OnValue>1</OnValue>
                <OffValue>0</OffValue>
            </Boolean>

            <Command Name="MyCommand">
                <pValue>Node</pValue>
                <CommandValue>10</CommandValue>
            </Command>

            <Enumeration Name="MyEnumeration">
                <EnumEntry Name="Entry0">
                    <Value>0</Value>
                    <NumericValue>1.0</NumericValue>
                    <NumericValue>10.0</NumericValue>
                    <IsSelfClearing>Yes</IsSelfClearing>
                </EnumEntry>
                <EnumEntry Name="Entry1">
                    <Value>1</Value>
                </EnumEntry>
                <pValue>MyNode</pValue>
            <PollingTime>10</PollingTime>
            </Enumeration>

            <Float Name="MyFloat">
                <Value>10.0</Value>
            </Float>

            <FloatReg Name="MyFloatReg">
              <Address>0x10000</Address>
              <Length>4</Length>
              <pPort>Device</pPort>
            </FloatReg>

            <StringReg Name="MyStringReg">
              <Address>100000</Address>
              <Length>128</Length>
              <pPort>Device</pPort>
            </StringReg>

            <Register Name="MyRegister">
              <Address>0x10000</Address>
              <Length>4</Length>
              <pPort>Device</pPort>
            </Register>

            <IntSwissKnife Name="MyIntSwissKnife">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <Constant Name="Const">10</Constant>
                <Expression Name="ConstBy2">2.0*Const</Expression>
                <Formula>Var1+Var2+ConstBy2</Formula>
             </IntSwissKnife>

            <SwissKnife Name="MySwissKnife">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <Constant Name="Const">INF</Constant>
                <Expression Name="ConstBy2">2.0*Const</Expression>
                <Formula>Var1+Var2+ConstBy2</Formula>
             </SwissKnife>

            <StructReg Comment="Struct Entry Comment">
                <Address>0x10000</Address>
                <Length>4</Length>
                <pPort>Device</pPort>
                <Endianess>BigEndian</Endianess>

                <StructEntry Name="MyStructEntry">
                    <Bit>24</Bit>
                </StructEntry>

            </StructReg>

        </RegisterDescription>
        "#;

        let document = xml::Document::from_str(xml).unwrap();
        let reg_desc: RegisterDescription = document.root_node().parse();

        assert_eq!(reg_desc.model_name(), "CameleonModel");
        assert_eq!(reg_desc.vendor_name(), "CameleonVendor");
        assert_eq!(reg_desc.standard_name_space(), StandardNameSpace::None);
        assert_eq!(reg_desc.schema_major_version(), 1);
        assert_eq!(reg_desc.schema_minor_version(), 1);
        assert_eq!(reg_desc.schema_subminor_version(), 0);
        assert_eq!(reg_desc.major_version(), 1);
        assert_eq!(reg_desc.minor_version(), 2);
        assert_eq!(reg_desc.subminor_version(), 3);
        assert_eq!(
            reg_desc.product_guid(),
            "01234567-0123-0123-0123-0123456789ab"
        );
        assert_eq!(
            reg_desc.version_guid(),
            "76543210-3210-3210-3210-ba9876543210"
        );
        assert_eq!(reg_desc.nodes().len(), 15);
    }
}
