use std::iter::FusedIterator;

use crate::RedBlackTree;

use super::{IntoIter, Range, RangeMut};

pub struct IntoValues<K, V>(IntoIter<K, V>);

impl<K: Ord, V> RedBlackTree<K, V> {
    /// Creates a consuming iterator visiting all the values, in order by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// a.insert(1, "hello");
    /// a.insert(2, "goodbye");
    ///
    /// let values: Vec<&str> = a.into_values().collect();
    /// assert_eq!(values, vec!["hello", "goodbye"]);
    /// ```
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues(self.into_iter())
    }

    /// Gets an iterator over the values of the map, in order by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// a.insert(1, "hello");
    /// a.insert(2, "goodbye");
    ///
    /// let values: Vec<_> = a.values().copied().collect();
    /// assert_eq!(values, ["hello", "goodbye"]);
    /// ```
    #[inline]
    pub fn values(&self) -> Values<K, V> {
        Values(self.into_iter(), self.len())
    }

    /// Gets a mutable iterator over the values of the map, in order by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// a.insert(1, String::from("hello"));
    /// a.insert(2, String::from("goodbye"));
    ///
    /// for value in a.values_mut() {
    ///     value.push('!') ;
    /// }
    ///
    /// let values: Vec<_> = a.values().cloned().collect();
    /// assert_eq!(values, [
    ///     String::from("hello!"),
    ///     String::from("goodbye!")
    /// ]);
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        let len = self.len();
        ValuesMut(self.into_iter(), len)
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

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

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| v)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

pub struct Values<'a, K, V>(Range<'a, K, V>, usize);

impl<'a, K: 'a + Ord, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| {
            self.1 -= 1;
            v
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, K: 'a + Ord, V: 'a> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| {
            self.1 -= 1;
            v
        })
    }
}

impl<'a, K: 'a + Ord, V: 'a> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.1
    }
}

impl<'a, K: 'a + Ord, V: 'a> FusedIterator for Values<'a, K, V> {}

pub struct ValuesMut<'a, K, V>(RangeMut<'a, K, V>, usize);

impl<'a, K: 'a + Ord, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| {
            self.1 -= 1;
            v
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, K: 'a + Ord, V: 'a> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, v)| {
            self.1 -= 1;
            v
        })
    }
}

impl<'a, K: 'a + Ord, V: 'a> ExactSizeIterator for ValuesMut<'a, K, V> {
    fn len(&self) -> usize {
        self.1
    }
}

impl<'a, K: 'a + Ord, V: 'a> FusedIterator for ValuesMut<'a, K, V> {}
