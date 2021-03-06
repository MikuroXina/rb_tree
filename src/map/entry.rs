use crate::RbTreeMap;

impl<K: Ord, V> RbTreeMap<K, V> {
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut count = RbTreeMap::new();
    ///
    /// for x in ["a", "b", "a", "c", "a", "b"] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count["a"], 3);
    /// assert_eq!(count["b"], 2);
    /// assert_eq!(count["c"], 1);
    /// ```
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        Entry { key, tree: self }
    }
}

#[derive(Debug)]
pub struct Entry<'a, K: Ord, V> {
    key: K,
    tree: &'a mut RbTreeMap<K, V>,
}

impl<'a, K: Ord, V> Entry<'a, K, V> {
    /// Returns a reference to this entry's key.
    #[inline]
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Ensures a value is in the entry by inserting `default` if empty, and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// ```
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        // Safety: The return value will not live longer than `tree`.
        unsafe {
            if self.tree.is_empty() || self.tree.root.search(&self.key).transpose().is_err() {
                self.tree
                    .root
                    .insert_node(self.key, default)
                    .unwrap_unchecked()
                    .value_mut()
            } else {
                self.tree.get_mut(&self.key).unwrap()
            }
        }
    }

    /// Ensures a value is in the entry by inserting the result of `default` function if empty, and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.entry("poneyland").or_insert_with(|| "hoho".to_string());
    ///
    /// assert_eq!(map["poneyland"], "hoho".to_string());
    /// ```
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        self.or_insert_with_key(move |_| default())
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of `default` function. This method allows for generating key-derived values for insertion by providing `default` a reference to the key that was moved during the `entry` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is unnecessary, unlike with [`or_insert_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    /// map.entry("poneyland").or_insert_with_key(|key| key.chars().count());
    ///
    /// assert_eq!(map["poneyland"], 9);
    /// ```
    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        // Safety: The return value will not live longer than `tree`.
        unsafe {
            if self.tree.is_empty() || self.tree.root.search(&self.key).transpose().is_err() {
                let value = default(&self.key);
                self.tree
                    .root
                    .insert_node(self.key, value)
                    .unwrap_unchecked()
                    .value_mut()
            } else {
                self.tree.get_mut(&self.key).unwrap()
            }
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map = RbTreeMap::new();
    ///
    /// map.entry("poneyland")
    ///     .and_modify(|e| *e += 1)
    ///     .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")
    ///     .and_modify(|e| *e += 1)
    ///     .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
    #[must_use]
    #[inline]
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        if let Some(entry) = self.tree.get_mut(&self.key) {
            f(entry);
        }
        self
    }

    /// Ensures a value is in the entry by inserting [`Default::default`] value if empty, and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RbTreeMap;
    ///
    /// let mut map: RbTreeMap<&str, Option<usize>> = RbTreeMap::new();
    /// map.entry("poneyland").or_default();
    ///
    /// assert_eq!(map["poneyland"], None);
    /// ```
    #[inline]
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }
}
