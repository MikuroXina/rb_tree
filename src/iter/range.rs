use std::{borrow, fmt, iter::FusedIterator, marker::PhantomData, ops};

use crate::{
    node::{ChildIndex, NodeRef},
    RedBlackTree,
};

use super::{MutLeafRange, RefLeafRange};

enum SearchBound<I> {
    Included(I),
    Excluded(I),
    AllIncluded,
    AllExcluded,
}

impl<I> From<ops::Bound<I>> for SearchBound<I> {
    fn from(bound: ops::Bound<I>) -> Self {
        match bound {
            ops::Bound::Included(idx) => Self::Included(idx),
            ops::Bound::Excluded(idx) => Self::Excluded(idx),
            ops::Bound::Unbounded => Self::AllIncluded,
        }
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    fn find_lower<I>(&self, bound: SearchBound<&I>) -> Option<NodeRef<K, V>>
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
    {
        match bound {
            SearchBound::Included(key) => {
                let mut current = self.root?;
                loop {
                    current = match key.cmp(current.key()) {
                        std::cmp::Ordering::Less => {
                            let left = current.child(ChildIndex::Left);
                            if left.is_none() {
                                break Some(current);
                            }
                            left.unwrap()
                        }
                        std::cmp::Ordering::Equal => break Some(current),
                        std::cmp::Ordering::Greater => {
                            let right = current.child(ChildIndex::Right);
                            if right.is_none() {
                                break Some(current);
                            }
                            right.unwrap()
                        }
                    }
                }
            }
            SearchBound::Excluded(key) => {
                let included_case = self.find_lower(SearchBound::Included(key))?;
                if key == included_case.key() {
                    Some(included_case)
                } else {
                    included_case
                        .child(ChildIndex::Right)
                        .or_else(|| included_case.parent())
                }
            }
            SearchBound::AllIncluded => self.first_node(),
            SearchBound::AllExcluded => None,
        }
    }

    fn find_upper<I>(&self, bound: SearchBound<&I>) -> Option<NodeRef<K, V>>
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
    {
        match bound {
            SearchBound::Included(key) => {
                let mut current = self.root?;
                loop {
                    current = match key.cmp(current.key()) {
                        std::cmp::Ordering::Less => {
                            let left = current.child(ChildIndex::Left);
                            if left.is_none() {
                                break Some(current);
                            }
                            left.unwrap()
                        }
                        std::cmp::Ordering::Equal => break Some(current),
                        std::cmp::Ordering::Greater => {
                            let right = current.child(ChildIndex::Right);
                            if right.is_none() {
                                break Some(current);
                            }
                            right.unwrap()
                        }
                    }
                }
            }
            SearchBound::Excluded(key) => {
                let included_case = self.find_upper(SearchBound::Included(key))?;
                if key == included_case.key() {
                    Some(included_case)
                } else {
                    included_case
                        .child(ChildIndex::Left)
                        .or_else(|| included_case.parent())
                }
            }
            SearchBound::AllIncluded => self.last_node(),
            SearchBound::AllExcluded => None,
        }
    }

    #[allow(clippy::type_complexity)]
    fn search_range<I, R>(&self, range: &R) -> (Option<NodeRef<K, V>>, Option<NodeRef<K, V>>)
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
        R: ops::RangeBounds<I>,
    {
        use SearchBound::*;
        let (start, end) = (range.start_bound().into(), range.end_bound().into());
        let (start, end) = match (start, end) {
            (Excluded(s), Excluded(e)) if s == e => (AllExcluded, AllExcluded),
            (Included(s) | Excluded(s), Included(e) | Excluded(e)) if s > e => {
                (AllExcluded, AllExcluded)
            }
            other => other,
        };
        let lower = self.find_lower(start);
        let upper = self.find_upper(end);
        (lower, upper)
    }

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
    pub fn range<I, R>(&self, range: R) -> Range<K, V>
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
        R: ops::RangeBounds<I>,
    {
        let (start, end) = self.search_range(&range);
        Range(RefLeafRange {
            start,
            end,
            _phantom: PhantomData,
        })
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
    pub fn range_mut<I, R>(&mut self, range: R) -> RangeMut<K, V>
    where
        I: Ord + ?Sized,
        K: borrow::Borrow<I>,
        R: ops::RangeBounds<I>,
    {
        let (start, end) = self.search_range(&range);
        RangeMut(MutLeafRange {
            start,
            end,
            _phantom: PhantomData,
        })
    }
}

pub struct Range<'a, K, V>(RefLeafRange<'a, K, V>);

impl<K, V> Clone for Range<'_, K, V> {
    fn clone(&self) -> Self {
        Self(RefLeafRange {
            start: self.0.start,
            end: self.0.end,
            _phantom: PhantomData,
        })
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Range<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K: 'a, V: 'a> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.cut_left()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.0.cut_right()
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(self) -> Option<Self::Item> {
        self.last()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Range<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.cut_right()
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for Range<'a, K, V> {}

pub struct RangeMut<'a, K, V>(MutLeafRange<'a, K, V>);

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for RangeMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Range(RefLeafRange {
            start: self.0.start,
            end: self.0.end,
            _phantom: PhantomData,
        })
        .fmt(f)
    }
}

impl<'a, K: 'a, V: 'a> Iterator for RangeMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.cut_left()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.0.cut_right()
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    fn max(self) -> Option<Self::Item> {
        self.last()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for RangeMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.cut_right()
    }
}

impl<'a, K: 'a, V: 'a> FusedIterator for RangeMut<'a, K, V> {}
