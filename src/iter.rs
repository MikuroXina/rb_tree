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

struct MutLeafRange<'a, K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a, K, V> MutLeafRange<'a, K, V> {
    fn cut_left(&mut self) -> Option<(&'a K, &'a mut V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent())?;
        self.start.replace(next).map(|p| p.key_value_mut())
    }

    fn cut_right(&mut self) -> Option<(&'a K, &'a mut V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent())?;
        self.end.replace(next).map(|p| p.key_value_mut())
    }
}

pub struct IterMut<'a, K, V> {
    range: MutLeafRange<'a, K, V>,
    length: usize,
}

impl<'a, K: 'a, V: 'a> IntoIterator for &'a mut RedBlackTree<K, V> {
    type Item = (&'a K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let start = self.first_node();
        let end = self.last_node();
        let length = self.len;
        IterMut {
            range: MutLeafRange {
                start,
                end,
                _phantom: PhantomData,
            },
            length,
        }
    }
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_left()
        }
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_right()
        }
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for IterMut<'a, K, V> {
    fn len(&self) -> usize {
        self.length
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for IterMut<'a, K, V> {}

// keys iterator

pub struct IntoKeys<K, V>(pub(super) IntoIter<K, V>);

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| k)
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize {
        self.0.length
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

pub struct Keys<'a, K, V>(pub(super) Iter<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| k)
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for Keys<'a, K, V> {
    fn len(&self) -> usize {
        self.0.length
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Keys<'a, K, V> {}

// values iterator

pub struct IntoValues<K, V>(pub(super) IntoIter<K, V>);

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| k)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.0.length
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

pub struct Values<'a, K, V>(pub(super) Iter<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.0.length
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Values<'a, K, V> {}

pub struct ValuesMut<'a, K, V>(pub(super) IterMut<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for ValuesMut<'a, K, V> {
    fn len(&self) -> usize {
        self.0.length
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for ValuesMut<'a, K, V> {}
