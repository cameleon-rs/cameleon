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
mod utils;
mod xml;

use group::GroupNode;
use struct_reg::StructRegNode;
use thiserror::Error;

use crate::{
    builder::{CacheStoreBuilder, NodeStoreBuilder, ValueStoreBuilder},
    store::NodeData,
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

pub fn parse(
    xml: &impl AsRef<str>,
    node_builder: &mut impl NodeStoreBuilder,
    value_builder: &mut impl ValueStoreBuilder,
    cache_builder: &mut impl CacheStoreBuilder,
) -> ParseResult<RegisterDescription> {
    let document = xml::Document::from_str(xml.as_ref())?;
    let mut node = document.root_node();
    let reg_desc = node.parse(node_builder, value_builder, cache_builder);
    while let Some(ref mut child) = node.next() {
        let children: Vec<NodeData> = child.parse(node_builder, value_builder, cache_builder);
        for child in children {
            let id = child.node_base().id();
            node_builder.store_node(id, child);
        }
    }

    Ok(reg_desc)
}

trait Parse {
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self;
}

impl Parse for Vec<NodeData> {
    fn parse(
        node: &mut xml::Node,
        node_builder: &mut impl NodeStoreBuilder,
        value_builder: &mut impl ValueStoreBuilder,
        cache_builder: &mut impl CacheStoreBuilder,
    ) -> Self {
        match node.tag_name() {
            NODE => vec![NodeData::Node(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            CATEGORY => vec![NodeData::Category(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            INTEGER => vec![NodeData::Integer(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            INT_REG => vec![NodeData::IntReg(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            MASKED_INT_REG => vec![NodeData::MaskedIntReg(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            BOOLEAN => vec![NodeData::Boolean(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            COMMAND => vec![NodeData::Command(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            ENUMERATION => vec![NodeData::Enumeration(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            FLOAT => vec![NodeData::Float(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            FLOAT_REG => vec![NodeData::FloatReg(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            STRING => vec![NodeData::String(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            STRING_REG => vec![NodeData::StringReg(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            REGISTER => vec![NodeData::Register(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            CONVERTER => vec![NodeData::Converter(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            INT_CONVERTER => vec![NodeData::IntConverter(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            SWISS_KNIFE => vec![NodeData::SwissKnife(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            INT_SWISS_KNIFE => vec![NodeData::IntSwissKnife(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            PORT => vec![NodeData::Port(Box::new(node.parse(
                node_builder,
                value_builder,
                cache_builder,
            )))],
            STRUCT_REG => {
                let node: StructRegNode = node.parse(node_builder, value_builder, cache_builder);
                node.into_masked_int_regs(cache_builder)
                    .into_iter()
                    .map(|node| NodeData::MaskedIntReg(node.into()))
                    .collect()
            }
            GROUP => {
                let node: GroupNode = node.parse(node_builder, value_builder, cache_builder);
                node.nodes
            }
            // TODO: Implement DCAM specific ndoes.
            CONF_ROM | TEXT_DESC | INT_KEY | ADV_FEATURE_LOCK | SMART_FEATURE => todo!(),
            _ => unreachable!(),
        }
    }
}
