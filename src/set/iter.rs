mod merge;

use self::merge::MergeIter;

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
        let (self_min, self_max) = if let Some(pair) = self.min().zip(self.max()) {
            pair
        } else {
            return Difference(DifferenceInner::Through(self.iter()));
        };
        let (other_min, other_max) = if let Some(pair) = other.min().zip(other.max()) {
            pair
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

    /// Visits the values representing the symmetric difference, i.e., the values that are in `self` or in `other` but not in both, in ascending order.
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
    /// let sym_diff: Vec<_> = a.symmetric_difference(&b).cloned().collect();
    /// assert_eq!(sym_diff, [1, 3]);
    /// ```
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, T>
    where
        T: Ord,
    {
        SymmetricDifference(MergeIter::new(self.iter(), other.iter()))
    }

    /// Visits the values representing the intersection,
    /// i.e., the values that are both in `self` and `other`,
    /// in ascending order.
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
    /// let intersection: Vec<_> = a.intersection(&b).cloned().collect();
    /// assert_eq!(intersection, [2]);
    /// ```
    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T>
    where
        T: Ord,
    {
        let (self_min, self_max) = if let Some(pair) = self.min().zip(self.max()) {
            pair
        } else {
            return Intersection(IntersectionInner::AtLeast(None));
        };
        let (other_min, other_max) = if let Some(pair) = other.min().zip(other.max()) {
            pair
        } else {
            return Intersection(IntersectionInner::AtLeast(None));
        };
        use std::cmp::Ordering::*;
        let inner = match (self_min.cmp(other_max), self_max.cmp(other_min)) {
            (Greater, _) | (_, Less) => IntersectionInner::AtLeast(None),
            (Equal, _) => IntersectionInner::AtLeast(Some(self_min)),
            (_, Equal) => IntersectionInner::AtLeast(Some(self_max)),
            _ if self.len() <= other.len() / ITER_PERFORMANCE_TIPPING_SIZE_DIFF => {
                IntersectionInner::Search {
                    small_iter: self.iter(),
                    large_set: other,
                }
            }
            _ if other.len() <= self.len() / ITER_PERFORMANCE_TIPPING_SIZE_DIFF => {
                IntersectionInner::Search {
                    small_iter: other.iter(),
                    large_set: self,
                }
            }
            _ => IntersectionInner::Stitch {
                a: self.iter(),
                b: other.iter(),
            },
        };
        Intersection(inner)
    }

    /// Visits the values representing the union,
    /// i.e., all the values in `self` or `other`, without duplicates,
    /// in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut a = RbTreeSet::new();
    /// a.insert(1);
    ///
    /// let mut b = RbTreeSet::new();
    /// b.insert(2);
    ///
    /// let union: Vec<_> = a.union(&b).cloned().collect();
    /// assert_eq!(union, [1, 2]);
    /// ```
    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, T>
    where
        T: Ord,
    {
        Union(MergeIter::new(self.iter(), other.iter()))
    }

    /// Returns `true` if `self` has no elements in common with `other`. This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let a: RbTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let mut b = RbTreeSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    pub fn is_disjoint(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        self.intersection(other).next().is_none()
    }

    /// Returns `true` if the set is a subset of another, i.e., `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let sup: RbTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let mut set = RbTreeSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    pub fn is_subset(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        if other.len() < self.len() {
            return false;
        }
        let (self_min, self_max) = if let Some(pair) = self.min().zip(self.max()) {
            pair
        } else {
            return true; // self is empty
        };
        let (other_min, other_max) = if let Some(pair) = other.min().zip(other.max()) {
            pair
        } else {
            return false; // other is empty
        };

        use std::cmp::Ordering::*;

        let mut self_iter = self.iter();
        match self_min.cmp(other_min) {
            Less => return false,
            Equal => {
                self_iter.next();
            }
            Greater => (),
        }
        match self_max.cmp(other_max) {
            Greater => return false,
            Equal => {
                self_iter.next_back();
            }
            Less => (),
        }
        if self_iter.len() <= other.len() / ITER_PERFORMANCE_TIPPING_SIZE_DIFF {
            for next in self_iter {
                if !other.contains(next) {
                    return false;
                }
            }
        } else {
            let mut other_iter = other.iter();
            other_iter.next();
            other_iter.next_back();
            let mut self_next = self_iter.next();
            while let Some(self1) = self_next {
                match other_iter.next().map_or(Less, |other1| self1.cmp(other1)) {
                    Less => return false,
                    Equal => self_next = self_iter.next(),
                    Greater => (),
                }
            }
        }
        true
    }

    /// Returns `true` if the set is a superset of another, i.e., `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let sub: RbTreeSet<_> = [1, 2].iter().cloned().collect();
    /// let mut set = RbTreeSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    pub fn is_superset(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        other.is_subset(self)
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
    /// iterates a small set, looks up in the large set
    Search {
        self_iter: Iter<'a, T>,
        other_set: &'a RbTreeSet<T>,
    },
    /// goes through the iterator
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

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<'a, T: Ord + 'a> FusedIterator for Difference<'a, T> {}

#[derive(Debug)]
pub struct SymmetricDifference<'a, T>(MergeIter<Iter<'a, T>>);

impl<T> Clone for SymmetricDifference<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T: Ord + 'a> Iterator for SymmetricDifference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (a_next, b_next) = self.0.nexts(Self::Item::cmp);
            if a_next.and(b_next).is_none() {
                return a_next.or(b_next);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lens = self.0.lens();
        (0, Some(lens.0 + lens.1))
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<T: Ord> FusedIterator for SymmetricDifference<'_, T> {}

#[derive(Debug)]
pub struct Intersection<'a, T>(IntersectionInner<'a, T>);

#[derive(Debug)]
enum IntersectionInner<'a, T> {
    /// iterate similarly sized sets jointly, spotting matches along the way
    Stitch { a: Iter<'a, T>, b: Iter<'a, T> },
    /// iterates a small set, looks up in the large set
    Search {
        small_iter: Iter<'a, T>,
        large_set: &'a RbTreeSet<T>,
    },
    /// returns a specific value or emptiness
    AtLeast(Option<&'a T>),
}

impl<T> Clone for Intersection<'_, T> {
    fn clone(&self) -> Self {
        Self(match &self.0 {
            IntersectionInner::Stitch { a, b } => IntersectionInner::Stitch {
                a: a.clone(),
                b: b.clone(),
            },
            IntersectionInner::Search {
                small_iter,
                large_set,
            } => IntersectionInner::Search {
                small_iter: small_iter.clone(),
                large_set,
            },
            IntersectionInner::AtLeast(opt) => IntersectionInner::AtLeast(*opt),
        })
    }
}

impl<'a, T: Ord + 'a> Iterator for Intersection<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        use std::cmp::Ordering::*;
        match &mut self.0 {
            IntersectionInner::Stitch { a, b } => {
                let (mut a_next, mut b_next) = (a.next()?, b.next()?);
                loop {
                    match a_next.cmp(b_next) {
                        Less => a_next = a.next()?,
                        Equal => return Some(a_next),
                        Greater => b_next = b.next()?,
                    }
                }
            }
            IntersectionInner::Search {
                small_iter,
                large_set,
            } => loop {
                let next = small_iter.next()?;
                if large_set.contains(next) {
                    return Some(next);
                }
            },
            IntersectionInner::AtLeast(opt) => opt.take(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            IntersectionInner::Stitch { a, b } => (0, Some(a.len().min(b.len()))),
            IntersectionInner::Search { small_iter, .. } => (0, Some(small_iter.len())),
            IntersectionInner::AtLeast(None) => (0, Some(0)),
            IntersectionInner::AtLeast(Some(_)) => (1, Some(1)),
        }
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<T: Ord> FusedIterator for Intersection<'_, T> {}

#[derive(Debug)]
pub struct Union<'a, T>(MergeIter<Iter<'a, T>>);

impl<T> Clone for Union<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T: Ord + 'a> Iterator for Union<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let (a_next, b_next) = self.0.nexts(Self::Item::cmp);
        a_next.or(b_next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lens = self.0.lens();
        (lens.0.max(lens.1), Some(lens.0 + lens.1))
    }

    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<T: Ord> FusedIterator for Union<'_, T> {}
