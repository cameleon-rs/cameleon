use super::{
    elem_type::{convert_to_int, StandardNameSpace},
    xml, CategoryNode, IntegerNode, Node,
};

pub struct RegisterDescription {
    model_name: String,
    vendor_name: String,
    tool_tip: Option<String>,
    standard_name_space: StandardNameSpace,
    schema_major_version: i64,
    schema_minor_version: i64,
    schema_subminor_version: i64,
    major_version: i64,
    minor_version: i64,
    subminor_version: i64,
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
        self.tool_tip.as_ref().map(|s| s.as_str())
    }

    pub fn standard_name_space(&self) -> StandardNameSpace {
        self.standard_name_space
    }

    pub fn schema_major_version(&self) -> i64 {
        self.schema_major_version
    }

    pub fn schema_subminor_version(&self) -> i64 {
        self.schema_subminor_version
    }

    pub fn schema_minor_version(&self) -> i64 {
        self.schema_minor_version
    }

    pub fn major_version(&self) -> i64 {
        self.major_version
    }

    pub fn minor_version(&self) -> i64 {
        self.minor_version
    }

    pub fn subminor_version(&self) -> i64 {
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

    pub(super) fn parse(mut node: xml::Node) -> Self {
        debug_assert!(node.tag_name() == "RegisterDescription");

        let model_name = node.attribute_of("ModelName").unwrap();

        let vendor_name = node.attribute_of("VendorName").unwrap();

        let tool_tip = node.attribute_of("ToolTip");

        let standard_name_space = node
            .attribute_of("StandardNameSpace")
            .unwrap()
            .as_str()
            .into();

        let schema_major_version =
            convert_to_int(&node.attribute_of("SchemaMajorVersion").unwrap());

        let schema_minor_version =
            convert_to_int(&node.attribute_of("SchemaMinorVersion").unwrap());

        let schema_subminor_version =
            convert_to_int(&node.attribute_of("SchemaSubMinorVersion").unwrap());

        let major_version = convert_to_int(&node.attribute_of("MajorVersion").unwrap());

        let minor_version = convert_to_int(&node.attribute_of("MinorVersion").unwrap());

        let subminor_version = convert_to_int(&node.attribute_of("SubMinorVersion").unwrap());

        let product_guid = node.attribute_of("ProductGuid").unwrap();

        let version_guid = node.attribute_of("VersionGuid").unwrap();

        let mut nodes = vec![];
        while let Some(child) = node.next() {
            let node = match child.tag_name().as_str() {
                "Node" => NodeKind::Node(Node::parse(child)),
                "Category" => NodeKind::Category(CategoryNode::parse(child)),
                "Integer" => NodeKind::Integer(IntegerNode::parse(child)),
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
    Node(Node),
    Category(CategoryNode),
    Integer(IntegerNode),
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
                <pFeature>MyInt</pFeature>
            </Category>

            <Integer Name="MyInt">
                <Value>10</Value>
            </Integer>

        </RegisterDescription>
        "#;

        let xml_parser = libxml::parser::Parser::default();
        let document = xml_parser.parse_string(xml).unwrap();

        let node = xml::Node::from_xmltree_node(document.get_root_element().unwrap());
        let reg_desc = RegisterDescription::parse(node);

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
        assert_eq!(reg_desc.nodes().len(), 2);
    }
}
