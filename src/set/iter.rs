use super::RbTreeSet;

use std::{
    borrow::Borrow,
    iter::{FusedIterator, Peekable},
    ops,
};

// This constant is used by functions that compare two sets.
//
// It's used to divide rather than multiply sizes, to rule out overflow, and it's a power of two to make that division cheap.
const ITER_PERFORMANCE_TIPPING_SIZE_DIFF: usize = 16;

impl<T> RbTreeSet<T> {
    /// Gets an iterator that visits the values in the BTreeSet in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let set: RbTreeSet<usize> = [1, 2, 3].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(&1));
    /// assert_eq!(set_iter.next(), Some(&2));
    /// assert_eq!(set_iter.next(), Some(&3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let set: RbTreeSet<usize> = [3, 1, 2].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(&1));
    /// assert_eq!(set_iter.next(), Some(&2));
    /// assert_eq!(set_iter.next(), Some(&3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter(self.map.keys())
    }

    /// Constructs a double-ended iterator over a sub-range of elements in the set.
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use std::ops::Bound::Included;
    ///
    /// let mut set = BTreeSet::new();
    /// set.insert(3);
    /// set.insert(5);
    /// set.insert(8);
    /// for &elem in set.range((Included(&4), Included(&8))) {
    ///     println!("{}", elem);
    /// }
    /// assert_eq!(Some(&5), set.range(4..).next());
    /// ```
    pub fn range<R, I>(&self, range: R) -> Range<T>
    where
        T: Ord + Borrow<I>,
        R: ops::RangeBounds<I>,
        I: Ord + ?Sized,
    {
        Range(self.map.range(range))
    }

    /// Visits the values representing the difference, i.e., the values that are in self but not in other, in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut a = RbTreeSet::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = RbTreeSet::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let diff: Vec<_> = a.difference(&b).cloned().collect();
    /// assert_eq!(diff, [1]);
    /// ```
    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<T>
    where
        T: Ord,
    {
        let self_root = self.map.root.inner();
        let (self_min, self_max) = if let Some((min, max)) = self_root
            .map(|r| r.min_child())
            .zip(self_root.map(|r| r.max_child()))
        {
            (min.key(), max.key())
        } else {
            return Difference(DifferenceInner::Through(self.iter()));
        };
        let other_root = other.map.root.inner();
        let (other_min, other_max) = if let Some((min, max)) = other_root
            .map(|r| r.min_child())
            .zip(other_root.map(|r| r.max_child()))
        {
            (min.key(), max.key())
        } else {
            return Difference(DifferenceInner::Through(self.iter()));
        };
        use std::cmp::Ordering::*;
        let inner = match (self_min.cmp(other_max), self_max.cmp(other_min)) {
            (Greater, _) | (_, Less) => DifferenceInner::Through(self.iter()),
            (Equal, _) => {
                let mut iter = self.iter();
                iter.next();
                DifferenceInner::Through(iter)
            }
            (_, Equal) => {
                let mut iter = self.iter();
                iter.next_back();
                DifferenceInner::Through(iter)
            }
            _ if self.len() <= other.len() / ITER_PERFORMANCE_TIPPING_SIZE_DIFF => {
                DifferenceInner::Search {
                    self_iter: self.iter(),
                    other_set: other,
                }
            }
            _ => DifferenceInner::Stitch {
                self_iter: self.iter(),
                other_iter: other.iter().peekable(),
            },
        };
        Difference(inner)
    }
}

#[derive(Debug)]
pub struct Iter<'a, T>(crate::map::iter::Keys<'a, T, ()>);

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

#[derive(Debug)]
pub struct Range<'a, T>(crate::map::iter::Range<'a, T, ()>);

impl<'a, T: 'a> Iterator for Range<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Range<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(k, _)| k)
    }
}

impl<'a, T: 'a> FusedIterator for Range<'a, T> {}

pub struct Difference<'a, T: 'a>(DifferenceInner<'a, T>);

#[derive(Debug)]
enum DifferenceInner<'a, T: 'a> {
    /// iterates all of `self_iter` and some of `other`, spotting matches along the way
    Stitch {
        self_iter: Iter<'a, T>,
        other_iter: Peekable<Iter<'a, T>>,
    },
    Search {
        self_iter: Iter<'a, T>,
        other_set: &'a RbTreeSet<T>,
    },
    Through(Iter<'a, T>),
}

impl<T> Clone for Difference<'_, T> {
    fn clone(&self) -> Self {
        Self(match &self.0 {
            DifferenceInner::Stitch {
                self_iter,
                other_iter,
            } => DifferenceInner::Stitch {
                self_iter: self_iter.clone(),
                other_iter: other_iter.clone(),
            },
            DifferenceInner::Search {
                self_iter,
                other_set,
            } => DifferenceInner::Search {
                self_iter: self_iter.clone(),
                other_set,
            },
            DifferenceInner::Through(iter) => DifferenceInner::Through(iter.clone()),
        })
    }
}

impl<'a, T: Ord + 'a> Iterator for Difference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        use std::cmp::Ordering::*;
        match &mut self.0 {
            DifferenceInner::Stitch {
                self_iter,
                other_iter,
            } => {
                let mut self_next = self_iter.next()?;
                loop {
                    match other_iter
                        .peek()
                        .map_or(Less, |other_next| self_next.cmp(other_next))
                    {
                        Less => return Some(self_next),
                        Equal => {
                            self_next = self_iter.next()?;
                        }
                        Greater => {}
                    }
                    other_iter.next();
                }
            }
            DifferenceInner::Search {
                self_iter,
                other_set,
            } => loop {
                let self_next = self_iter.next()?;
                if !other_set.contains(self_next) {
                    return Some(self_next);
                }
            },
            DifferenceInner::Through(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (self_len, other_len) = match &self.0 {
            DifferenceInner::Stitch {
                self_iter,
                other_iter,
            } => (self_iter.len(), other_iter.len()),
            DifferenceInner::Search {
                self_iter,
                other_set,
            } => (self_iter.len(), other_set.len()),
            DifferenceInner::Through(iter) => (iter.len(), 0),
        };
        (self_len.saturating_sub(other_len), Some(self_len))
    }

    fn min(mut self) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: Ord,
    {
        self.next()
    }
}

impl<'a, T: Ord + 'a> FusedIterator for Difference<'a, T> {}
