use super::{elem_name::*, elem_type::*, node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct StringNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,
    streamable: bool,
    value: ImmOrPNode<String>,
}

impl StringNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn value(&self) -> &ImmOrPNode<String> {
        &self.value
    }
}

impl Parse for StringNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), STRING);

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while(P_INVALIDATOR);
        let streamable = node.parse_if(STREAMABLE).unwrap_or_default();
        let value = if let Some(next_node) = node.next_if(VALUE) {
            ImmOrPNode::Imm(next_node.text().into())
        } else {
            ImmOrPNode::PNode(node.next_text().unwrap().into())
        };

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_with_imm() {
        let xml = r#"
        <String Name="TestNode">
            <Streamable>Yes</Streamable>
            <Value>Immediate String</Value>
        </String>
        "#;

        let node: StringNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.streamable(), true);
        assert_eq!(node.value(), &ImmOrPNode::Imm("Immediate String".into()));
    }

    #[test]
    fn test_string_with_p_node() {
        let xml = r#"
        <String Name="TestNode">
            <pValue>AnotherStringNode</pValue>
        </String>
        "#;

        let node: StringNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.streamable(), false);
        assert_eq!(node.value(), &ImmOrPNode::PNode("AnotherStringNode".into()));
    }
}
