use std::iter::Peekable;

pub(super) struct Node<'a, 'input> {
    inner: roxmltree::Node<'a, 'input>,
    children: Peekable<roxmltree::Children<'a, 'input>>,
    attributes: Attributes<'a, 'input>,
}

impl<'a, 'input> Node<'a, 'input> {
    pub(super) fn from_xmltree_node(node: roxmltree::Node<'a, 'input>) -> Self {
        let children = node.children().peekable();
        let attributes = Attributes::from_xmltree_attrs(node.attributes());

        Self {
            inner: node,
            children,
            attributes,
        }
    }

    pub(super) fn next(&mut self) -> Option<Self> {
        let node = self.peek()?;
        self.children.next();

        Some(node)
    }

    pub(super) fn next_if(&mut self, tag_name: &str) -> Option<Self> {
        let next_node = self.peek()?;
        if next_node.tag_name() == tag_name {
            self.next()
        } else {
            None
        }
    }

    pub(super) fn next_text_if(&mut self, tag_name: &str) -> Option<&'a str> {
        let next = self.next_if(tag_name)?;
        Some(next.text())
    }

    pub(super) fn peek(&mut self) -> Option<Self> {
        let mut inner;
        loop {
            inner = self.children.peek()?;
            if inner.node_type() != roxmltree::NodeType::Element {
                self.children.next();
            } else {
                break;
            }
        }
        let node = Self::from_xmltree_node(*inner);

        Some(node)
    }

    pub(super) fn tag_name(&self) -> &str {
        self.inner.tag_name().name()
    }

    pub(super) fn attribute_of(&self, name: &str) -> Option<&str> {
        self.attributes.attribute_of(name)
    }

    pub(super) fn text(&self) -> &'a str {
        self.inner.text().unwrap()
    }
}

struct Attributes<'a, 'input> {
    attrs: &'a [roxmltree::Attribute<'input>],
}

impl<'a, 'input> Attributes<'a, 'input> {
    fn from_xmltree_attrs(attrs: &'a [roxmltree::Attribute<'input>]) -> Self {
        Self { attrs }
    }

    fn attribute_of(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|attr| attr.name() == name)
            .map(|attr| attr.value())
    }
}
