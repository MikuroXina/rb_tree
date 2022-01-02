use std::{borrow, fmt, iter::FusedIterator, marker::PhantomData, ops};

use crate::RedBlackTree;

use super::RefLeafRange;

impl<K: Ord, V> RedBlackTree<K, V> {
    /// Constructs a double-ended iterator over a sub-range of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    /// use std::ops::Bound::Included;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(3, "a");
    /// map.insert(5, "b");
    /// map.insert(8, "c");
    /// for (&key, &value) in map.range((Included(&4), Included(&8))) {
    ///     println!("{}: {}", key, value);
    /// }
    /// assert_eq!(map.range(4..).next(), Some((&5, &"b")));
    /// ```
    pub fn range<I, R>(&self, range: R) -> Range<K, V, R>
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
        R: ops::RangeBounds<I>,
    {
        Range(RefLeafRange::new(self, range), PhantomData)
    }

    /// Constructs a mutable double-ended iterator over a sub-range of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map: RedBlackTree<&str, i32> = ["Alice", "Bob", "Carol", "Cheryl"]
    ///     .into_iter()
    ///     .map(|s| (s, 0))
    ///     .collect();
    /// for (_, balance) in map.range_mut("B".."Cheryl") {
    ///     *balance += 100;
    /// }
    /// for (name, balance) in &map {
    ///     println!("{} => {}", name, balance);
    /// }
    /// ```
    pub fn range_mut<I, R>(&mut self, range: R) -> RangeMut<K, V, R>
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
        R: ops::RangeBounds<I>,
    {
        RangeMut(RefLeafRange::new(self, range), PhantomData)
    }
}

pub struct Range<'a, K, V, R>(RefLeafRange<K, V, R>, PhantomData<&'a ()>);

impl<K, V, R: Clone> Clone for Range<'_, K, V, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<K: fmt::Debug, V: fmt::Debug, R> fmt::Debug for Range<'_, K, V, R>
where
    K: Ord,
    R: Clone + ops::RangeBounds<K>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V, R> Iterator for Range<'a, K, V, R>
where
    K: Ord + 'a,
    V: 'a,
    R: ops::RangeBounds<K>,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.cut_left().map(|n| n.key_value())
    }

    fn last(mut self) -> Option<Self::Item> {
        self.0.cut_right().map(|n| n.key_value())
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(self) -> Option<Self::Item> {
        self.last()
    }
}

impl<'a, K, V, R> DoubleEndedIterator for Range<'a, K, V, R>
where
    K: Ord + 'a,
    V: 'a,
    R: ops::RangeBounds<K>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.cut_right().map(|n| n.key_value())
    }
}

impl<'a, K, V, R> FusedIterator for Range<'a, K, V, R>
where
    K: Ord + 'a,
    V: 'a,
    R: ops::RangeBounds<K>,
{
}

pub struct RangeMut<'a, K, V, I>(RefLeafRange<K, V, I>, PhantomData<&'a mut ()>);

impl<K: fmt::Debug, V: fmt::Debug, R> fmt::Debug for RangeMut<'_, K, V, R>
where
    K: Ord,
    R: Clone + ops::RangeBounds<K>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Range(self.0.clone(), PhantomData).fmt(f)
    }
}

impl<'a, K, V, R> Iterator for RangeMut<'a, K, V, R>
where
    K: Ord + 'a,
    V: 'a,
    R: ops::RangeBounds<K>,
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.cut_left().map(|n| n.key_value_mut())
    }

    fn last(mut self) -> Option<Self::Item> {
        self.0.cut_right().map(|n| n.key_value_mut())
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(self) -> Option<Self::Item> {
        self.last()
    }
}

impl<'a, K, V, R> DoubleEndedIterator for RangeMut<'a, K, V, R>
where
    K: Ord + 'a,
    V: 'a,
    R: ops::RangeBounds<K>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.cut_right().map(|n| n.key_value_mut())
    }
}

impl<'a, K, V, R> FusedIterator for RangeMut<'a, K, V, R>
where
    K: Ord + 'a,
    V: 'a,
    R: ops::RangeBounds<K>,
{
}
