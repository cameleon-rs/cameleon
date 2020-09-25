use std::iter::Peekable;
use std::vec::IntoIter;

use libxml::tree::node::Node as XmlNode;

pub(super) struct Node {
    inner: XmlNode,
    children: Peekable<IntoIter<XmlNode>>,
}

impl Node {
    pub(super) fn from_xmltree_node(node: XmlNode) -> Self {
        let children = node.get_child_elements().into_iter().peekable();

        Self {
            inner: node,
            children,
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

    pub(super) fn next_text_if(&mut self, tag_name: &str) -> Option<String> {
        let next_node = self.next_if(tag_name)?;
        Some(next_node.text())
    }

    pub(super) fn peek(&mut self) -> Option<Self> {
        let node = self.children.peek()?;
        let node = Self::from_xmltree_node(node.clone());

        Some(node)
    }

    pub(super) fn tag_name(&self) -> String {
        self.inner.get_name()
    }

    pub(super) fn text(&self) -> String {
        self.inner.get_content()
    }

    pub(super) fn attribute_of(&self, name: &str) -> Option<String> {
        self.inner.get_attribute(name)
    }
}
