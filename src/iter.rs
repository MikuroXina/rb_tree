use crate::{
    node::{ChildIndex, Node, NodeRef},
    RedBlackTree,
};

struct LeafRange<K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
}

impl<K, V> LeafRange<K, V> {
    fn advance_left(&mut self) -> Option<Box<Node<K, V>>> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent())?;
        self.start
            .replace(next)
            .map(|p| unsafe { Box::from_raw(p.as_raw().as_ptr()) })
    }

    fn advance_right(&mut self) -> Option<Box<Node<K, V>>> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent())?;
        self.end
            .replace(next)
            .map(|p| unsafe { Box::from_raw(p.as_raw().as_ptr()) })
    }
}

pub struct IntoIter<K, V> {
    tree: RedBlackTree<K, V>,
    range: LeafRange<K, V>,
    length: usize,
}

impl<K, V> IntoIterator for RedBlackTree<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let start = self.first_node();
        let end = self.last_node();
        let length = self.len;
        IntoIter {
            tree: self,
            range: LeafRange { start, end },
            length,
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.advance_left().map(|p| p.into_element())
        }
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.advance_right().map(|p| p.into_element())
        }
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.length
    }
}
