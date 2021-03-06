use std::iter::FusedIterator;

use crate::RbTreeMap;

use super::{IntoIter, Iter};

impl<K, V> RbTreeMap<K, V> {
    /// Creates a consuming iterator visiting all the keys, in sorted order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
    /// a.insert(2, "b");
    /// a.insert(1, "a");
    ///
    /// let mut keys = a.into_keys();
    /// assert_eq!(keys.next(), Some(1));
    /// assert_eq!(keys.next(), Some(2));
    /// assert_eq!(keys.next(), None);
    /// ```
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys(self.into_iter())
    }

    /// Gets an iterator over the keys of the map, in sorted order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
    /// a.insert(2, "b");
    /// a.insert(1, "a");
    ///
    /// let keys: Vec<i32> = a.keys().copied().collect();
    /// assert_eq!(keys, [1, 2]);
    /// ```
    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        Keys(self.into_iter(), self.len())
    }
}

#[derive(Debug)]
pub struct IntoKeys<K, V>(IntoIter<K, V>);

impl<K, V> Iterator for IntoKeys<K, V> {
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

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| k)
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

#[derive(Debug)]
pub struct Keys<'a, K, V>(Iter<'a, K, V>, usize);

impl<K, V> Clone for Keys<'_, K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| {
            self.1 -= 1;
            k
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| {
            self.1 -= 1;
            k
        })
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for Keys<'a, K, V> {
    fn len(&self) -> usize {
        self.1
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Keys<'a, K, V> {}
