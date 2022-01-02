use std::iter::FusedIterator;

use crate::RedBlackTree;

use super::{LeafRange, Range, RangeMut};

pub struct IntoIter<K, V> {
    range: LeafRange<K, V>,
    length: usize,
}

impl<K: Ord, V> RedBlackTree<K, V> {
    /// Gets an iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// a.insert(3, "c");
    /// a.insert(2, "b");
    /// a.insert(1, "a");
    ///
    /// let mut iter = a.iter();
    /// assert_eq!(iter.next(), Some((&1, &"a")));
    /// assert_eq!(iter.next(), Some((&2, &"b")));
    /// assert_eq!(iter.next(), Some((&3, &"c")));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Range<K, V> {
        self.range(..)
    }

    /// Gets a iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for (key, value) in map.iter_mut() {
    ///     if key != &"a" {
    ///         *value += 10;
    ///     }
    /// }
    ///
    /// assert_eq!(map[&"a"], 1);
    /// assert_eq!(map[&"b"], 12);
    /// assert_eq!(map[&"c"], 13);
    /// ```
    pub fn iter_mut(&mut self) -> RangeMut<K, V> {
        self.range_mut(..)
    }
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }

    fn last(mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_right()
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

impl<'a, K: Ord, V> IntoIterator for &'a RedBlackTree<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Range<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.range(..)
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a mut RedBlackTree<K, V> {
    type Item = (&'a K, &'a mut V);

    type IntoIter = RangeMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.range_mut(..)
    }
}
