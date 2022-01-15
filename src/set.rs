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

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let set: RbTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.get(&2), Some(&2));
    /// assert_eq!(set.get(&4), None);
    /// ```
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.get_key_value(value).map(|(k, _)| k)
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

    /// Adds a value to the set, replacing the existing value, if any, that is equal to the given one. Returns the replaced value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    /// set.insert(Vec::<i32>::new());
    ///
    /// assert_eq!(set.get(&[][..]).unwrap().capacity(), 0);
    /// set.replace(Vec::with_capacity(10));
    /// assert_eq!(set.get(&[][..]).unwrap().capacity(), 10);
    /// ```
    pub fn replace(&mut self, value: T) -> Option<T>
    where
        T: Ord,
    {
        self.map.insert(value, ()).map(|(k, _)| k)
    }

    /// Removes a value from the set. Returns whether the value was present in the set.
    ///
    /// The value may be any borrowed form of the set’s value type, but the ordering on the borrowed form must match the ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.remove(value).is_some()
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// The value may be any borrowed form of the set’s value type, but the ordering on the borrowed form must match the ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set: RbTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.take(&2), Some(2));
    /// assert_eq!(set.take(&2), None);
    /// ```
    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.remove_entry(value).map(|(k, _)| k)
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut v = RbTreeSet::new();
    /// v.insert(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a reference to the first value in the set, if any. This value is always the minimum of all values in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    /// assert_eq!(set.first(), None);
    /// set.insert(1);
    /// assert_eq!(set.first(), Some(&1));
    /// set.insert(2);
    /// assert_eq!(set.first(), Some(&1));
    /// ```
    pub fn first<Q>(&self) -> Option<&Q>
    where
        T: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.first().map(|(k, _)| k.borrow())
    }

    /// Returns a reference to the last value in the set, if any. This value is always the maximum of all values in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    /// assert_eq!(set.last(), None);
    /// set.insert(1);
    /// assert_eq!(set.last(), Some(&1));
    /// set.insert(2);
    /// assert_eq!(set.last(), Some(&2));
    /// ```
    pub fn last<Q>(&self) -> Option<&Q>
    where
        T: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.last().map(|(k, _)| k.borrow())
    }

    /// Removes the first value from the set and returns it, if any. The first value is always the minimum value in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_first() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    pub fn pop_first(&mut self) -> Option<T>
    where
        T: Ord,
    {
        self.map.pop_first().map(|(k, _)| k)
    }

    /// Removes the last value from the set and returns it, if any. The last value is always the maximum value in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set = RbTreeSet::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_last() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    pub fn pop_last(&mut self) -> Option<T>
    where
        T: Ord,
    {
        self.map.pop_last().map(|(k, _)| k)
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
