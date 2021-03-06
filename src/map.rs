pub mod entry;
pub mod iter;

use crate::node::Root;

use std::{borrow::Borrow, fmt, hash, ops};

/// A map based on a red-black tree.
pub struct RbTreeMap<K, V> {
    pub(crate) root: Root<K, V>,
}

impl<K, V> Drop for RbTreeMap<K, V> {
    fn drop(&mut self) {
        // Safety: `self` will not be used after.
        unsafe { drop(std::ptr::read(self).into_iter()) }
    }
}

impl<K: fmt::Debug + Ord, V: fmt::Debug> fmt::Debug for RbTreeMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for RbTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for RbTreeMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut tree = Self::new();
        for (k, v) in iter {
            tree.insert(k, v);
        }
        tree
    }
}

impl<K: Ord, V> Extend<(K, V)> for RbTreeMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K: Ord + Copy + 'a, V: Copy + 'a> Extend<(&'a K, &'a V)> for RbTreeMap<K, V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(*k, *v);
        }
    }
}

impl<K: hash::Hash, V: hash::Hash> hash::Hash for RbTreeMap<K, V> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.root.len().hash(state);
        self.iter().for_each(|e| e.hash(state));
    }
}

impl<K, Q, V> ops::Index<&'_ Q> for RbTreeMap<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, index: &'_ Q) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<K, Q, V> ops::IndexMut<&'_ Q> for RbTreeMap<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    fn index_mut(&mut self, index: &'_ Q) -> &mut Self::Output {
        self.get_mut(index).expect("no entry found for key")
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for RbTreeMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.root.len() == other.root.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<K: Eq, V: Eq> Eq for RbTreeMap<K, V> {}

impl<K: PartialOrd, V: PartialOrd> PartialOrd for RbTreeMap<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<K: Ord, V: Ord> Ord for RbTreeMap<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

unsafe impl<K: Send, V: Send> Send for RbTreeMap<K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for RbTreeMap<K, V> {}

impl<K, V> RbTreeMap<K, V> {
    /// Creates an empty `RbTreeMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    ///
    /// map.insert(1, "a");
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self { root: Root::new() }
    }

    /// Removes all elements from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Returns whether the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.root.len()
    }
}

impl<K: Ord, V> RbTreeMap<K, V> {
    /// Moves all elements from `other` into `Self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut a = RbTreeMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    ///
    /// let mut b = RbTreeMap::new();
    /// b.insert(3, "d");
    /// b.insert(4, "e");
    /// b.insert(5, "f");
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// let drained: Vec<_> = a.into_iter().collect();
    /// assert_eq!(drained, vec![
    ///     (1, "a"),
    ///     (2, "b"),
    ///     (3, "d"),
    ///     (4, "e"),
    ///     (5, "f"),
    /// ]);
    /// ```
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            std::mem::swap(self, other);
            return;
        }

        for (k, v) in other.drain_filter(|_, _| true) {
            self.insert(k, v);
        }
    }

    /// Inserts a key-value pair into the map. Then the old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::<i32, &str>::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some((37, "b")));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        self.root.insert_node(key, value).err()
    }

    /// Removes a key from the map, returning the old value if the key was in.
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Removes a key from the map, returning the old key-value pair if the key was in.
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove_entry(&1), None);
    /// ```
    #[inline]
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root.remove_node(key)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }

    /// Returns a mutable reference ti the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    #[inline]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root
            .search(key)?
            .ok()
            .map(|n| unsafe { n.value_mut() })
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    #[inline]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root
            .search(key)?
            .ok()
            .map(|n| unsafe { n.key_value() })
    }

    /// Returns whether the map contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Retains only the elements specified by the predicate. In other words, remove all pairs `(k, v)` such that the predicate `f(&k, &mut v)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map: RbTreeMap<i32, i32> = (0..8).map(|x| (x, x * 10)).collect();
    /// map.retain(|&k, _| k % 2 == 0);
    /// assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![(0, 0), (2, 20), (4, 40), (6, 60)]);
    /// ```
    #[inline]
    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, mut f: F) {
        self.drain_filter(move |k, v| !f(k, v));
    }

    /// Returns the first key-value pair in the map. The key in this pair is the minimum key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// assert_eq!(map.first(), None);
    /// map.insert(1, "b");
    /// map.insert(2, "a");
    /// assert_eq!(map.first(), Some((&1, &"b")));
    /// ```
    pub fn first(&self) -> Option<(&K, &V)> {
        Some(unsafe { self.root.inner()?.min_child().key_value() })
    }

    /// Returns the last key-value pair in the map. The key in this pair is the maximum key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "b");
    /// map.insert(2, "a");
    /// assert_eq!(map.last(), Some((&2, &"a")));
    /// ```
    pub fn last(&self) -> Option<(&K, &V)> {
        Some(unsafe { self.root.inner()?.max_child().key_value() })
    }

    pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        Some(unsafe { self.root.inner()?.min_child().key_value_mut() })
    }

    pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
        Some(unsafe { self.root.inner()?.max_child().key_value_mut() })
    }

    /// Removes and returns the first element in the map. The key of this element is the minimum key that was in the map.
    ///
    /// # Examples
    ///
    /// Draining elements in ascending order, while keeping a usable map each iteration.
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// while let Some((key, _val)) = map.pop_first() {
    ///     assert!(map.iter().all(|(k, _v)| *k > key));
    /// }
    /// assert!(map.is_empty());
    /// ```
    pub fn pop_first(&mut self) -> Option<(K, V)> {
        self.root.remove_min()
    }

    /// Removes and returns the last element in the map. The key of this element is the maximum key that was in the map.
    ///
    /// # Examples
    ///
    /// Draining elements in descending order, while keeping a usable map each iteration.
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// while let Some((key, _val)) = map.pop_last() {
    ///     assert!(map.iter().all(|(k, _v)| *k < key));
    /// }
    /// assert!(map.is_empty());
    /// ```
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        self.root.remove_max()
    }
}
