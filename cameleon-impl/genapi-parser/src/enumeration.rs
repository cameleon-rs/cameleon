use super::{elem_name::*, elem_type::*, node_base::*, xml, Parse};

#[derive(Debug, Clone)]
pub struct EnumerationNode {
    attr_base: NodeAttributeBase,
    elem_base: NodeElementBase,

    p_invalidators: Vec<String>,
    streamable: bool,
    entries: Vec<EnumEntryNode>,
    value: ImmOrPNode<i64>,
    p_selected: Vec<String>,
    polling_time: Option<u64>,
}

impl EnumerationNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn streamable(&self) -> bool {
        self.streamable
    }

    pub fn entries(&self) -> &[EnumEntryNode] {
        &self.entries
    }

    pub fn value(&self) -> &ImmOrPNode<i64> {
        &self.value
    }

    pub fn p_selected(&self) -> &[String] {
        &self.p_selected
    }

    pub fn polling_time(&self) -> Option<u64> {
        self.polling_time
    }
}

impl Parse for EnumerationNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), ENUMERATION);

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while(P_INVALIDATOR);
        let streamable = node.parse_if(STREAMABLE).unwrap_or_default();
        let mut entries = vec![];
        while let Some(mut ent_node) = node.next_if(ENUM_ENTRY) {
            entries.push(ent_node.parse());
        }
        let value = node.parse();
        let p_selected = node.parse_while(P_SELECTED);
        let polling_time = node.parse_if(POLLING_TIME);

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

    p_invalidators: Vec<String>,
    value: i64,
    numeric_values: Vec<f64>,
    symbolic: Option<String>,
    is_self_clearing: bool,
}

impl EnumEntryNode {
    pub fn node_base(&self) -> NodeBase<'_> {
        NodeBase::new(&self.attr_base, &self.elem_base)
    }

    pub fn p_invalidators(&self) -> &[String] {
        &self.p_invalidators
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn numeric_values(&self) -> &[f64] {
        &self.numeric_values
    }

    pub fn symbolic(&self) -> Option<&str> {
        self.symbolic.as_deref()
    }

    pub fn set_symbolic(&mut self, s: String) {
        self.symbolic = Some(s)
    }

    pub fn is_self_clearing(&self) -> bool {
        self.is_self_clearing
    }
}

impl Parse for EnumEntryNode {
    fn parse(node: &mut xml::Node) -> Self {
        debug_assert_eq!(node.tag_name(), ENUM_ENTRY);

        let attr_base = node.parse();
        let elem_base = node.parse();

        let p_invalidators = node.parse_while(P_INVALIDATOR);
        let value = node.parse();
        let numeric_values = node.parse_while(NUMERIC_VALUE);
        let symbolic = node.parse_if(SYMBOLIC);
        let is_self_clearing = node.parse_if(IS_SELF_CLEARING).unwrap_or_default();

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

        let node: EnumerationNode = xml::Document::from_str(&xml).unwrap().root_node().parse();

        assert_eq!(node.value(), &ImmOrPNode::PNode("MyNode".into()));
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
