mod drain;
mod keys;
mod leaf;
mod range;
mod values;

pub use drain::*;
pub use keys::*;
pub use leaf::*;
pub use range::*;
pub use values::*;

use std::{iter::FusedIterator, marker::PhantomData};

use crate::RbTreeMap;

#[derive(Debug, Clone, Copy)]
enum PreviousStep {
    Parent,
    LeftChild,
    RightChild,
}

impl PreviousStep {
    fn is_left_child(self) -> bool {
        matches!(self, Self::LeftChild)
    }

    fn is_right_child(self) -> bool {
        matches!(self, Self::RightChild)
    }
}

pub struct IntoIter<K, V> {
    range: DyingLeafRange<K, V>,
    length: usize,
}

#[derive(Debug)]
pub struct Iter<'a, K, V> {
    range: RefLeafRange<K, V>,
    length: usize,
    _phantom: PhantomData<(&'a K, &'a V)>,
}

#[derive(Debug)]
pub struct IterMut<'a, K, V> {
    range: RefLeafRange<K, V>,
    length: usize,
    _phantom: PhantomData<(&'a K, &'a mut V)>,
}

impl<K, V> RbTreeMap<K, V> {
    /// Gets an iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
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
    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        let length = self.root.len();
        Iter {
            range: RefLeafRange::all(self),
            length,
            _phantom: PhantomData,
        }
    }

    /// Gets a iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
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
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        let length = self.root.len();
        IterMut {
            range: RefLeafRange::all(self),
            length,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> IntoIterator for RbTreeMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let length = self.root.len();
        IntoIter {
            range: DyingLeafRange::new(self),
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

impl<'a, K, V> IntoIterator for &'a RbTreeMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            length: self.length,
            _phantom: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_left().map(|n| unsafe { n.key_value() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_right().map(|n| unsafe { n.key_value() })
        }
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.length
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K, V> IntoIterator for &'a mut RbTreeMap<K, V> {
    type Item = (&'a K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_left().map(|n| unsafe { n.key_value_mut() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            self.range.cut_right().map(|n| unsafe { n.key_value_mut() })
        }
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.length
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}
