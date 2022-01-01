use std::iter::FusedIterator;

use crate::RedBlackTree;

use super::{IntoIter, Iter, IterMut};

pub struct IntoValues<K, V>(IntoIter<K, V>);

impl<K, V> RedBlackTree<K, V> {
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues(self.into_iter())
    }

    pub fn values(&self) -> Values<K, V> {
        Values(self.into_iter())
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut(self.into_iter())
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| k)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

pub struct Values<'a, K, V>(pub(super) Iter<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Values<'a, K, V> {}

pub struct ValuesMut<'a, K, V>(pub(super) IterMut<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| v)
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for ValuesMut<'a, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for ValuesMut<'a, K, V> {}
