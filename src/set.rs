pub mod iter;

use crate::RbTreeMap;

use std::{borrow::Borrow, fmt};

/// A set based on a red-black tree.
pub struct RbTreeSet<T> {
    map: RbTreeMap<T, ()>,
}

impl<T> Default for RbTreeSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for RbTreeSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T> RbTreeSet<T> {
    /// Creates a new, empty `RbTreeSet`. Does not allocate anything on its own.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let set: RbTreeSet<i32> = RbTreeSet::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            map: RbTreeMap::new(),
        }
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut v = RbTreeSet::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub const fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut v = RbTreeSet::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns true if the set contains a value.
    ///
    /// The value may be any borrowed form of the set’s value type, but the ordering on the borrowed form must match the ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let set: RbTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, true is returned.
    ///
    /// If the set did have this value present, false is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, value: T) -> bool
    where
        T: Ord,
    {
        self.map.insert(value, ()).is_none()
    }
}

impl<T: Ord> FromIterator<T> for RbTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        for item in iter {
            set.insert(item);
        }
        set
    }
}