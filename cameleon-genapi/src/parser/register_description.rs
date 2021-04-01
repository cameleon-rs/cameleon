use super::{
    convert_to_uint,
    elem_name::{
        ADV_FEATURE_LOCK, BOOLEAN, CATEGORY, COMMAND, CONF_ROM, CONVERTER, ENUMERATION, FLOAT,
        FLOAT_REG, GROUP, INTEGER, INT_CONVERTER, INT_KEY, INT_REG, INT_SWISS_KNIFE, MAJOR_VERSION,
        MASKED_INT_REG, MINOR_VERSION, MODEL_NAME, NODE, PORT, PRODUCT_GUID, REGISTER,
        REGISTER_DESCRIPTION, SCHEMA_MAJOR_VERSION, SCHEMA_MINOR_VERSION, SCHEMA_SUB_MINOR_VERSION,
        SMART_FEATURE, STANDARD_NAME_SPCACE, STRING, STRING_REG, STRUCT_REG, SUB_MINOR_VERSION,
        SWISS_KNIFE, TEXT_DESC, TOOL_TIP, VENDOR_NAME, VERSION_GUID,
    },
    elem_type::StandardNameSpace,
    node_base::NodeBase,
    node_store::NodeStore,
    xml, BooleanNode, CategoryNode, CommandNode, ConverterNode, EnumerationNode, FloatNode,
    FloatRegNode, GroupNode, IntConverterNode, IntRegNode, IntSwissKnifeNode, IntegerNode,
    MaskedIntRegNode, Node, Parse, PortNode, RegisterNode, StringNode, StringRegNode,
    StructRegNode, SwissKnifeNode,
};

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
}

impl RegisterDescription {
    #[must_use]
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    #[must_use]
    pub fn vendor_name(&self) -> &str {
        &self.vendor_name
    }

    #[must_use]
    pub fn tool_tip(&self) -> Option<&str> {
        self.tool_tip.as_deref()
    }

    #[must_use]
    pub fn standard_name_space(&self) -> StandardNameSpace {
        self.standard_name_space
    }

    #[must_use]
    pub fn schema_major_version(&self) -> u64 {
        self.schema_major_version
    }

    #[must_use]
    pub fn schema_subminor_version(&self) -> u64 {
        self.schema_subminor_version
    }

    #[must_use]
    pub fn schema_minor_version(&self) -> u64 {
        self.schema_minor_version
    }

    #[must_use]
    pub fn major_version(&self) -> u64 {
        self.major_version
    }

    #[must_use]
    pub fn minor_version(&self) -> u64 {
        self.minor_version
    }

    #[must_use]
    pub fn subminor_version(&self) -> u64 {
        self.subminor_version
    }

    #[must_use]
    pub fn product_guid(&self) -> &str {
        &self.product_guid
    }

    #[must_use]
    pub fn version_guid(&self) -> &str {
        &self.version_guid
    }
}

impl Parse for RegisterDescription {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
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

        while let Some(ref mut child) = node.next() {
            for child in child.parse::<Vec<NodeData>>(store) {
                let id = child.node_base().id();
                store.store_node(id, child);
            }
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeData {
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
    String(Box<StringNode>),
    StringReg(Box<StringRegNode>),
    Register(Box<RegisterNode>),
    Converter(Box<ConverterNode>),
    IntConverter(Box<IntConverterNode>),
    SwissKnife(Box<SwissKnifeNode>),
    IntSwissKnife(Box<IntSwissKnifeNode>),
    Port(Box<PortNode>),

    // TODO: Implement DCAM specific ndoes.
    ConfRom(()),
    TextDesc(()),
    IntKey(()),
    AdvFeatureLock(()),
    SmartFeature(()),
}

impl NodeData {
    pub fn node_base(&self) -> NodeBase<'_> {
        match self {
            Self::Node(node) => node.node_base(),
            Self::Category(node) => node.node_base(),
            Self::Integer(node) => node.node_base(),
            Self::IntReg(node) => node.node_base(),
            Self::MaskedIntReg(node) => node.node_base(),
            Self::Boolean(node) => node.node_base(),
            Self::Command(node) => node.node_base(),
            Self::Enumeration(node) => node.node_base(),
            Self::Float(node) => node.node_base(),
            Self::FloatReg(node) => node.node_base(),
            Self::String(node) => node.node_base(),
            Self::StringReg(node) => node.node_base(),
            Self::Register(node) => node.node_base(),
            Self::Converter(node) => node.node_base(),
            Self::IntConverter(node) => node.node_base(),
            Self::SwissKnife(node) => node.node_base(),
            Self::IntSwissKnife(node) => node.node_base(),
            Self::Port(node) => node.node_base(),
            _ => todo!(),
        }
    }
}

impl Parse for Vec<NodeData> {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Vec<NodeData> {
        match node.tag_name() {
            NODE => vec![NodeData::Node(Box::new(node.parse(store)))],
            CATEGORY => vec![NodeData::Category(Box::new(node.parse(store)))],
            INTEGER => vec![NodeData::Integer(Box::new(node.parse(store)))],
            INT_REG => vec![NodeData::IntReg(Box::new(node.parse(store)))],
            MASKED_INT_REG => vec![NodeData::MaskedIntReg(Box::new(node.parse(store)))],
            BOOLEAN => vec![NodeData::Boolean(Box::new(node.parse(store)))],
            COMMAND => vec![NodeData::Command(Box::new(node.parse(store)))],
            ENUMERATION => vec![NodeData::Enumeration(Box::new(node.parse(store)))],
            FLOAT => vec![NodeData::Float(Box::new(node.parse(store)))],
            FLOAT_REG => vec![NodeData::FloatReg(Box::new(node.parse(store)))],
            STRING => vec![NodeData::String(Box::new(node.parse(store)))],
            STRING_REG => vec![NodeData::StringReg(Box::new(node.parse(store)))],
            REGISTER => vec![NodeData::Register(Box::new(node.parse(store)))],
            CONVERTER => vec![NodeData::Converter(Box::new(node.parse(store)))],
            INT_CONVERTER => vec![NodeData::IntConverter(Box::new(node.parse(store)))],
            SWISS_KNIFE => vec![NodeData::SwissKnife(Box::new(node.parse(store)))],
            INT_SWISS_KNIFE => vec![NodeData::IntSwissKnife(Box::new(node.parse(store)))],
            PORT => vec![NodeData::Port(Box::new(node.parse(store)))],
            STRUCT_REG => node
                .parse::<StructRegNode>(store)
                .to_masked_int_regs()
                .into_iter()
                .map(|node| NodeData::MaskedIntReg(node.into()))
                .collect(),
            GROUP => node.parse::<GroupNode>(store).nodes,

            // TODO: Implement DCAM specific ndoes.
            CONF_ROM | TEXT_DESC | INT_KEY | ADV_FEATURE_LOCK | SMART_FEATURE => todo!(),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
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
                <pFeature>MyString</pFeature>
                <pFeature>MyStringReg</pFeature>
                <pFeature>MyRegister</pFeature>
                <pFeature>MyConverter</pFeature>
                <pFeature>MyIntConverter</pFeature>
                <pFeature>MySwissKnife</pFeature>
                <pFeature>MyIntSwissKnife</pFeature>
                <pFeature>MyPort</pFeature>
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

            <String Name="MyString">
                <Streamable>Yes</Streamable>
                <Value>Immediate String</Value>
            </String>

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

            <Converter Name="MyConverter">
                <pVariable Name="Var1">pValue1</pVariable>
                <pVariable Name="Var2">pValue2</pVariable>
                <FormulaTo>FROM*Var1/Var2</FormulaTo>
                <FormulaFrom>TO/Var1*Var2</FormulaFrom>
                <pValue>Target</pValue>
             </Converter>

            <IntConverter Name="MyIntConverter">
                <FormulaTo>FROM</FormulaTo>
                <FormulaFrom>TO</FormulaFrom>
                <pValue>Target</pValue>
             </IntConverter>

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

            <Port Name="MyPort">
                <ChunkID>Fd3219</ChunkID>
                <SwapEndianess>Yes</SwapEndianess>
            </Port>

            <StructReg Comment="Struct Entry Comment">
                <Address>0x10000</Address>
                <Length>4</Length>
                <pPort>Device</pPort>
                <Endianess>BigEndian</Endianess>

                <StructEntry Name="MyStructEntry">
                    <Bit>24</Bit>
                </StructEntry>
            </StructReg>

            <Group Comment="Nothing to say">
                <IntReg Name="RegImpl">
                  <Address>0x10000</Address>
                  <pLength>LengthNode</pLength>
                  <pPort>Device</pPort>
                </IntReg>
            </Group>


        </RegisterDescription>
        "#;

        let document = xml::Document::from_str(xml).unwrap();
        let mut store = NodeStore::new();
        let reg_desc: RegisterDescription = document.root_node().parse(&mut store);

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
    }
}
