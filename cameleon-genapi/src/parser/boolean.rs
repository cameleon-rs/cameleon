use super::{
    elem_name::{BOOLEAN, OFF_VALUE, ON_VALUE, P_INVALIDATOR, P_SELECTED, STREAMABLE},
    elem_type::ImmOrPNode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct BooleanNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    streamable: bool,
    value: ImmOrPNode<bool>,
    on_value: Option<i64>,
    off_value: Option<i64>,
    p_selected: Vec<NodeId>,
}

impl BooleanNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn streamable(&self) -> bool {
        self.streamable
    }

    #[must_use]
    pub fn value(&self) -> &ImmOrPNode<bool> {
        &self.value
    }

    #[must_use]
    pub fn on_value(&self) -> Option<i64> {
        self.on_value
    }

    #[must_use]
    pub fn off_value(&self) -> Option<i64> {
        self.off_value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }
}

impl Parse for BooleanNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), BOOLEAN);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let value = node.parse(store);
        let on_value = node.parse_if(ON_VALUE, store);
        let off_value = node.parse_if(OFF_VALUE, store);
        let p_selected = node.parse_while(P_SELECTED, store);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            value,
            on_value,
            off_value,
            p_selected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_node_with_p_node() {
        let xml = r#"
            <Boolean Name="TestNode">
                <pValue>Node</pValue>
                <OnValue>1</OnValue>
                <OffValue>0</OffValue>
            </Boolean>
            "#;

        let mut store = NodeStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);
        assert_eq!(node.value(), &ImmOrPNode::PNode(store.id_by_name("Node")));
        assert_eq!(node.on_value(), Some(1));
        assert_eq!(node.off_value(), Some(0));
    }

    #[test]
    fn test_boolean_node_with_imm() {
        let xml1 = r#"
            <Boolean Name="TestNode">
                <Value>true</Value>
            </Boolean>
            "#;

        let mut store = NodeStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml1)
            .unwrap()
            .root_node()
            .parse(&mut store);
        assert_eq!(node.value(), &ImmOrPNode::Imm(true));

        let xml2 = r#"
            <Boolean Name="TestNode">
                <Value>false</Value>
            </Boolean>
            "#;

        let mut store2 = NodeStore::new();
        let node: BooleanNode = xml::Document::from_str(&xml2)
            .unwrap()
            .root_node()
            .parse(&mut store2);
        assert_eq!(node.value(), &ImmOrPNode::Imm(false));
    }
}
