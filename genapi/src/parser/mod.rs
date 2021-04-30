mod boolean;
mod category;
mod command;
mod converter;
mod elem_name;
mod elem_type;
mod enumeration;
mod float;
mod float_reg;
mod formula;
mod group;
mod int_converter;
mod int_reg;
mod int_swiss_knife;
mod integer;
mod masked_int_reg;
mod node;
mod node_base;
mod port;
mod register;
mod register_base;
mod register_description;
mod string;
mod string_reg;
mod struct_reg;
mod swiss_knife;
mod xml;

use group::GroupNode;
use struct_reg::StructRegNode;
use thiserror::Error;

use crate::{
    store::{DefaultNodeStore, DefaultValueStore, NodeData, WritableNodeStore, ValueStore},
    RegisterDescription,
};

use elem_name::{
    ADV_FEATURE_LOCK, BOOLEAN, CATEGORY, COMMAND, CONF_ROM, CONVERTER, ENUMERATION, FLOAT,
    FLOAT_REG, GROUP, INTEGER, INT_CONVERTER, INT_KEY, INT_REG, INT_SWISS_KNIFE, MASKED_INT_REG,
    NODE, PORT, REGISTER, SMART_FEATURE, STRING, STRING_REG, STRUCT_REG, SWISS_KNIFE, TEXT_DESC,
};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("encodings must be UTF8: {}", 0)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("invalid XML syntax: {}", 0)]
    InvalidSyntax(#[from] roxmltree::Error),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;

pub struct Parser<'a> {
    document: xml::Document<'a>,
}

impl<'a> Parser<'a> {
    pub fn from_bytes(input: &'a impl AsRef<[u8]>) -> ParseResult<Self> {
        let input = std::str::from_utf8(input.as_ref())?;
        let document = xml::Document::from_str(input)?;
        Ok(Self { document })
    }

    pub fn parse(&self) -> ParseResult<(RegisterDescription, DefaultNodeStore, DefaultValueStore)> {
        let mut node_store = DefaultNodeStore::new();
        let mut value_store = DefaultValueStore::new();
        let reg_desc = self.parse_with_store(&mut node_store, &mut value_store)?;
        Ok((reg_desc, node_store, value_store))
    }

    pub fn parse_with_store<T, U>(
        &self,
        mut node_store: T,
        mut value_store: U,
    ) -> ParseResult<RegisterDescription>
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        let mut node = self.document.root_node();
        let reg_desc = node.parse(&mut node_store, &mut value_store);
        while let Some(ref mut child) = node.next() {
            let children: Vec<NodeData> = child.parse(&mut node_store, &mut value_store);
            for child in children {
                let id = child.node_base().id();
                node_store.store_node(id, child);
            }
        }

        Ok(reg_desc)
    }

    #[must_use]
    pub fn inner_str(&self) -> &'a str {
        self.document.inner_str()
    }
}

trait Parse {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore;
}

impl Parse for Vec<NodeData> {
    fn parse<T, U>(node: &mut xml::Node, node_store: &mut T, value_store: &mut U) -> Self
    where
        T: WritableNodeStore,
        U: ValueStore,
    {
        match node.tag_name() {
            NODE => vec![NodeData::Node(Box::new(
                node.parse(node_store, value_store),
            ))],
            CATEGORY => vec![NodeData::Category(Box::new(
                node.parse(node_store, value_store),
            ))],
            INTEGER => vec![NodeData::Integer(Box::new(
                node.parse(node_store, value_store),
            ))],
            INT_REG => vec![NodeData::IntReg(Box::new(
                node.parse(node_store, value_store),
            ))],
            MASKED_INT_REG => vec![NodeData::MaskedIntReg(Box::new(
                node.parse(node_store, value_store),
            ))],
            BOOLEAN => vec![NodeData::Boolean(Box::new(
                node.parse(node_store, value_store),
            ))],
            COMMAND => vec![NodeData::Command(Box::new(
                node.parse(node_store, value_store),
            ))],
            ENUMERATION => vec![NodeData::Enumeration(Box::new(
                node.parse(node_store, value_store),
            ))],
            FLOAT => vec![NodeData::Float(Box::new(
                node.parse(node_store, value_store),
            ))],
            FLOAT_REG => vec![NodeData::FloatReg(Box::new(
                node.parse(node_store, value_store),
            ))],
            STRING => vec![NodeData::String(Box::new(
                node.parse(node_store, value_store),
            ))],
            STRING_REG => vec![NodeData::StringReg(Box::new(
                node.parse(node_store, value_store),
            ))],
            REGISTER => vec![NodeData::Register(Box::new(
                node.parse(node_store, value_store),
            ))],
            CONVERTER => vec![NodeData::Converter(Box::new(
                node.parse(node_store, value_store),
            ))],
            INT_CONVERTER => vec![NodeData::IntConverter(Box::new(
                node.parse(node_store, value_store),
            ))],
            SWISS_KNIFE => vec![NodeData::SwissKnife(Box::new(
                node.parse(node_store, value_store),
            ))],
            INT_SWISS_KNIFE => vec![NodeData::IntSwissKnife(Box::new(
                node.parse(node_store, value_store),
            ))],
            PORT => vec![NodeData::Port(Box::new(
                node.parse(node_store, value_store),
            ))],
            STRUCT_REG => {
                let node: StructRegNode = node.parse(node_store, value_store);
                node.into_masked_int_regs()
                    .into_iter()
                    .map(|node| NodeData::MaskedIntReg(node.into()))
                    .collect()
            }
            GROUP => {
                let node: GroupNode = node.parse(node_store, value_store);
                node.nodes
            }
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
    fn test_parser() {
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
        let parser = Parser::from_bytes(&xml).unwrap();
        parser.parse().unwrap();
    }
}
