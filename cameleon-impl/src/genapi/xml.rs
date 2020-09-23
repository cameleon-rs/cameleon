use std::iter::Peekable;

use super::{GenApiError, GenApiResult, Span};

pub(super) struct Node<'a, 'input> {
    inner: roxmltree::Node<'a, 'input>,
    children: Peekable<roxmltree::Children<'a, 'input>>,
    attributes: Attributes<'a, 'input>,
}

impl<'a, 'input> Node<'a, 'input> {
    pub(super) fn from_xmltree_node(node: roxmltree::Node<'a, 'input>) -> Span<Self> {
        let children = node.children().peekable();
        let attributes = Attributes::from_xmltree_attrs(node.attributes());
        let range = node.range();

        Span::from_range(
            Self {
                inner: node,
                children,
                attributes,
            },
            range,
        )
    }

    pub(super) fn next_child(&mut self) -> Option<Span<Self>> {
        let node = self.peek_child()?;
        self.children.next();

        Some(node)
    }

    pub(super) fn next_child_if(&mut self, tag_name: &str) -> Option<Span<Self>> {
        let next_node = self.peek_child()?;
        if next_node.tag_name() == tag_name {
            self.next_child()
        } else {
            None
        }
    }

    pub(super) fn peek_child(&mut self) -> Option<Span<Self>> {
        let inner = self.children.peek()?;
        let node = Self::from_xmltree_node(*inner);

        Some(node)
    }

    pub(super) fn next_attribute(&mut self) -> Option<Span<Attribute<'_>>> {
        self.attributes.next_attribute()
    }

    pub(super) fn next_attribute_if(&mut self, name: &str) -> Option<Span<Attribute<'_>>> {
        let next_attr = self.peek_attribute()?;
        if *next_attr.name() == name {
            self.next_attribute()
        } else {
            None
        }
    }

    pub(super) fn peek_attribute(&self) -> Option<Span<Attribute<'_>>> {
        self.attributes.peek_attribute()
    }

    pub(super) fn tag_name(&self) -> &str {
        self.inner.tag_name().name()
    }

    pub(super) fn text(&self) -> Option<Span<&'a str>> {
        match self.inner.first_child() {
            Some(node) if node.node_type() == roxmltree::NodeType::Text => {
                let text_node = self.inner.first_child().unwrap();
                let range = text_node.range();
                let text = text_node.text().unwrap();
                Some(Span::from_range(text, range))
            }
            _ => None,
        }
    }

    pub(super) fn expect_text(&self) -> GenApiResult<Span<&'a str>> {
        self.text().ok_or_else(|| {
            GenApiError::ElementIsEmpty(Span::from_range(
                self.tag_name().into(),
                self.inner.range(),
            ))
        })
    }

    pub(super) fn next_child_text_if(
        &mut self,
        tag_name: &str,
    ) -> GenApiResult<Option<Span<&'a str>>> {
        if let Some(node) = self.next_child_if(tag_name) {
            let text = node.expect_text()?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }
}

pub(super) struct Attribute<'a> {
    name: Span<&'a str>,
    value: Span<&'a str>,
}

impl<'a> Attribute<'a> {
    pub(super) fn name(&self) -> Span<&str> {
        self.name
    }

    pub(super) fn value(&self) -> Span<&str> {
        self.value
    }
}

struct Attributes<'a, 'input> {
    attrs: &'a [roxmltree::Attribute<'input>],
    cur: usize,
}

impl<'a, 'input> Attributes<'a, 'input> {
    fn from_xmltree_attrs(attrs: &'a [roxmltree::Attribute<'input>]) -> Self {
        Self { attrs, cur: 0 }
    }

    fn next_attribute(&mut self) -> Option<Span<Attribute<'_>>> {
        let cur = self.cur;

        if self.cur < self.attrs.len() {
            self.cur += 1;
        }

        self.attribute_of(cur)
    }

    fn peek_attribute(&self) -> Option<Span<Attribute<'_>>> {
        self.attribute_of(self.cur)
    }

    fn attribute_of(&self, index: usize) -> Option<Span<Attribute<'_>>> {
        if index == self.attrs.len() {
            return None;
        }

        let attr = &self.attrs[index];
        let name = attr.name();
        let value = attr.value();
        let value_range = attr.value_range();
        let name_range = attr.range();

        let attr_range = name_range.start..value_range.end;

        Some(Span::from_range(
            Attribute {
                name: Span::from_range(name, name_range),
                value: Span::from_range(value, value_range),
            },
            attr_range,
        ))
    }
}
