use super::{
    elem_name::{
        ENUMERATION, ENUM_ENTRY, IS_SELF_CLEARING, NUMERIC_VALUE, POLLING_TIME, P_INVALIDATOR,
        P_SELECTED, STREAMABLE, SYMBOLIC,
    },
    elem_type::ImmOrPNode,
    node_base::{NodeAttributeBase, NodeBase, NodeElementBase},
    node_store::{NodeId, NodeStore},
    xml, Parse,
};

#[derive(Debug, Clone)]
pub struct EnumerationNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    streamable: bool,
    entries: Vec<EnumEntryNode>,
    value: ImmOrPNode<i64>,
    p_selected: Vec<NodeId>,
    polling_time: Option<u64>,
}

impl EnumerationNode {
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
    pub fn entries(&self) -> &[EnumEntryNode] {
        &self.entries
    }

    #[must_use]
    pub fn value(&self) -> &ImmOrPNode<i64> {
        &self.value
    }

    #[must_use]
    pub fn p_selected(&self) -> &[NodeId] {
        &self.p_selected
    }

    #[must_use]
    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

impl Parse for EnumerationNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), ENUMERATION);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let streamable = node.parse_if(STREAMABLE, store).unwrap_or_default();
        let mut entries = vec![];
        while let Some(mut ent_node) = node.next_if(ENUM_ENTRY) {
            entries.push(ent_node.parse(store));
        }
        let value = node.parse(store);
        let p_selected = node.parse_while(P_SELECTED, store);
        let polling_time = node.parse_if(POLLING_TIME, store);

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            streamable,
            entries,
            value,
            p_selected,
            polling_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumEntryNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<NodeId>,
    value: i64,
    numeric_values: Vec<f64>,
    symbolic: Option<String>,
    is_self_clearing: bool,
}

impl EnumEntryNode {
    #[must_use]
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    #[must_use]
    pub fn p_invalidators(&self) -> &[NodeId] {
        &self.p_invalidators
    }

    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }

    #[must_use]
    pub fn numeric_values(&self) -> &[f64] {
        &self.numeric_values
    }

    #[must_use]
    pub fn symbolic(&self) -> Option<&str> {
        self.symbolic.as_deref()
    }

    pub fn set_symbolic(&mut self, s: String) {
        self.symbolic = Some(s)
    }

    #[must_use]
    pub fn is_self_clearing(&self) -> bool {
        self.is_self_clearing
    }
}

impl Parse for EnumEntryNode {
    fn parse(node: &mut xml::Node, store: &mut NodeStore) -> Self {
        debug_assert_eq!(node.tag_name(), ENUM_ENTRY);

        let attr_base = node.parse(store);
        let elem_base = node.parse(store);

        let p_invalidators = node.parse_while(P_INVALIDATOR, store);
        let value = node.parse(store);
        let numeric_values = node.parse_while(NUMERIC_VALUE, store);
        let symbolic = node.parse_if(SYMBOLIC, store);
        let is_self_clearing = node.parse_if(IS_SELF_CLEARING, store).unwrap_or_default();

        Self {
            attr_base,
            elem_base,
            p_invalidators,
            value,
            numeric_values,
            symbolic,
            is_self_clearing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumeration() {
        let xml = r#"
            <Enumeration Name="TestNode">
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
            "#;

        let mut store = NodeStore::new();
        let node: EnumerationNode = xml::Document::from_str(&xml)
            .unwrap()
            .root_node()
            .parse(&mut store);

        assert_eq!(node.value(), &ImmOrPNode::PNode(store.id_by_name("MyNode")));
        assert_eq!(node.polling_time(), Some(10));

        let entries = node.entries();
        assert_eq!(entries.len(), 2);

        let entry0 = &entries[0];
        assert_eq!(entry0.value(), 0);
        assert!((entry0.numeric_values()[0] - 1.0).abs() < f64::EPSILON);
        assert!((entry0.numeric_values()[1] - 10.0).abs() < f64::EPSILON);
        assert_eq!(entry0.is_self_clearing(), true);

        let entry1 = &entries[1];
        assert_eq!(entry1.value(), 1);
        assert_eq!(entry1.is_self_clearing(), false);
    }
}
