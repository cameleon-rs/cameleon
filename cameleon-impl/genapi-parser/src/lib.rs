mod category;
mod elem_type;
mod integer;
mod node;
mod node_base;
mod register_description;
mod xml;

pub use category::*;
pub use integer::*;
pub use node::*;
pub use node_base::*;
pub use register_description::*;

use thiserror::Error;

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

    /// Verify input xml with GenApi schema.
    // TODO:
    pub fn verify_xml(&self) -> ParseResult<()> {
        todo!()
    }

    pub fn parse(&self) -> ParseResult<RegisterDescription> {
        // TODO:
        // self.verify_xml()?;
        Ok(self.document.root_node().parse())
    }

    pub fn inner_str(&self) -> &'a str {
        self.document.inner_str()
    }
}

trait Parse {
    fn parse(node: &mut xml::Node) -> Self;
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
