use std::{iter::FusedIterator, marker::PhantomData};

use crate::{
    node::{ChildIndex, NodeRef},
    RedBlackTree,
};

struct LeafRange<K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
}

impl<K, V> LeafRange<K, V> {
    fn cut_left(&mut self) -> Option<(K, V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent())?;
        self.start.replace(next).map(|p| unsafe { p.deallocate() })
    }

    fn cut_right(&mut self) -> Option<(K, V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent())?;
        self.end.replace(next).map(|p| unsafe { p.deallocate() })
    }
}

pub struct IntoIter<K, V> {
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
        std::mem::forget(self);
        IntoIter {
            range: LeafRange { start, end },
            length,
        }
    }
}

impl<K, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_left()
        }
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_right()
        }
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.length
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

struct RefLeafRange<'a, K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, K, V> RefLeafRange<'a, K, V> {
    fn cut_left(&mut self) -> Option<(&'a K, &'a V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent())?;
        self.start.replace(next).map(|p| p.key_value())
    }

    fn cut_right(&mut self) -> Option<(&'a K, &'a V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent())?;
        self.end.replace(next).map(|p| p.key_value())
    }
}

pub struct Iter<'a, K, V> {
    range: RefLeafRange<'a, K, V>,
    length: usize,
}

impl<'a, K: 'a, V: 'a> IntoIterator for &'a RedBlackTree<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let start = self.first_node();
        let end = self.last_node();
        let length = self.len;
        Iter {
            range: RefLeafRange {
                start,
                end,
                _phantom: PhantomData,
            },
            length,
        }
    }
}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            range: RefLeafRange {
                start: self.range.start,
                end: self.range.end,
                _phantom: PhantomData,
            },
            length: self.length,
        }
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_left()
        }
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_right()
        }
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for Iter<'a, K, V> {
    fn len(&self) -> usize {
        self.length
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Iter<'a, K, V> {}
