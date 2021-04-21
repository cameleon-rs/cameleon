use std::iter::Peekable;

use crate::store::NodeStore;

use super::{Parse, ParseResult};

pub(super) struct Document<'input> {
    document: roxmltree::Document<'input>,
}

impl<'input> Document<'input> {
    pub(super) fn from_str(s: &'input str) -> ParseResult<Self> {
        let document = roxmltree::Document::parse(s)?;
        Ok(Self { document })
    }

    pub(super) fn root_node<'a>(&'a self) -> Node<'a, 'input> {
        let root = self.document.root_element();
        Node::from_xmltree_node(root)
    }

    pub(super) fn inner_str(&self) -> &'input str {
        self.document.input_text()
    }
}

pub(super) struct Node<'a, 'input> {
    inner: roxmltree::Node<'a, 'input>,
    children: Peekable<roxmltree::Children<'a, 'input>>,
    attributes: Attributes<'a, 'input>,
}

impl<'a, 'input> Node<'a, 'input> {
    pub(super) fn parse<T, U>(&mut self, store: &mut U) -> T
    where
        T: Parse,
        U: NodeStore,
    {
        T::parse(self, store)
    }

    pub(super) fn parse_if<T, U>(&mut self, tag_name: &str, store: &mut U) -> Option<T>
    where
        T: Parse,
        U: NodeStore,
    {
        if self.peek()?.tag_name() == tag_name {
            Some(self.parse(store))
        } else {
            None
        }
    }

    pub(super) fn parse_while<T, U>(&mut self, tag_name: &str, store: &mut U) -> Vec<T>
    where
        T: Parse,
        U: NodeStore,
    {
        let mut res = vec![];
        while let Some(parsed) = self.parse_if(tag_name, store) {
            res.push(parsed);
        }
        res
    }

    pub(super) fn next(&mut self) -> Option<Self> {
        let node = self.peek()?;
        self.children.next();

        Some(node)
    }

    pub(super) fn next_if(&mut self, tag_name: &str) -> Option<Self> {
        if self.peek()?.tag_name() == tag_name {
            self.next()
        } else {
            None
        }
    }

    pub(super) fn next_text(&mut self) -> Option<&'a str> {
        Some(self.next()?.text())
    }

    pub(super) fn peek(&mut self) -> Option<Self> {
        let mut inner;
        loop {
            inner = self.children.peek()?;
            if inner.node_type() == roxmltree::NodeType::Element {
                break;
            }
            self.children.next();
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

    fn from_xmltree_node(node: roxmltree::Node<'a, 'input>) -> Self {
        debug_assert!(node.node_type() == roxmltree::NodeType::Element);
        let children = node.children().peekable();
        let attributes = Attributes::from_xmltree_attrs(node.attributes());

        Self {
            inner: node,
            children,
            attributes,
        }
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
        self.attrs.iter().find_map(|attr| {
            if attr.name() == name {
                Some(roxmltree::Attribute::value(attr))
            } else {
                None
            }
        })
    }
}
