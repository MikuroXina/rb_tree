mod balance;
pub mod entry;
pub mod iter;
mod mem;
mod node;
#[cfg(test)]
mod tests;

use node::{ChildIndex, Node, NodeRef};

use std::{borrow::Borrow, fmt, hash, marker::PhantomData, ops};

pub struct RedBlackTree<K, V> {
    root: Option<NodeRef<K, V>>,
    len: usize,
    _phantom: PhantomData<Box<Node<K, V>>>,
}

// private methods
impl<K, V> RedBlackTree<K, V> {
    fn first_node(&self) -> Option<NodeRef<K, V>> {
        let mut current = self.root?;
        while let Some(left) = current.left() {
            current = left;
        }
        Some(current)
    }

    fn last_node(&self) -> Option<NodeRef<K, V>> {
        let mut current = self.root?;
        while let Some(right) = current.right() {
            current = right;
        }
        Some(current)
    }
}

// private methods
impl<K: Ord, V> RedBlackTree<K, V> {
    fn insert_node(&mut self, new_node: NodeRef<K, V>, (target, idx): (NodeRef<K, V>, ChildIndex)) {
        target.set_child(idx, Some(new_node));

        self.balance_after_insert(new_node);
    }

    fn remove_node(&mut self, node: NodeRef<K, V>) -> (K, V) {
        let child = match node.children() {
            (Some(_), Some(right)) => {
                let mut min_in_right = right;
                while let Some(min) = min_in_right.left() {
                    min_in_right = min;
                }
                while min_in_right.index_on_parent().unwrap().is_left() {
                    self.rotate(min_in_right, ChildIndex::Left);
                    min_in_right = min_in_right.parent().unwrap();
                }
                Some(min_in_right)
            }
            (None, None) => {
                // node is root
                return unsafe { self.root.take().unwrap().deallocate() };
            }
            (l, r) => l.xor(r),
        };
        debug_assert!(child.and_then(|c| c.left()).is_none());

        self.balance_after_remove(node);

        if let Some(parent) = node.parent() {
            parent.set_child(node.index_on_parent().unwrap(), child);
        }
        if let Some(child) = child {
            child.set_child(ChildIndex::Left, node.left());
        }
        unsafe { node.deallocate() }
    }

    fn search_node<Q>(&self, key: &Q) -> Result<NodeRef<K, V>, (NodeRef<K, V>, ChildIndex)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root.unwrap().search(key)
    }
}

impl<K, V> Drop for RedBlackTree<K, V> {
    fn drop(&mut self) {
        drop(unsafe { std::ptr::read(self) }.into_iter());
    }
}

impl<K: fmt::Debug + Ord, V: fmt::Debug> fmt::Debug for RedBlackTree<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for RedBlackTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for RedBlackTree<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut tree = Self::new();
        for (k, v) in iter {
            tree.insert(k, v);
        }
        tree
    }
}

impl<K: Ord, V> Extend<(K, V)> for RedBlackTree<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K: Ord + Copy + 'a, V: Copy + 'a> Extend<(&'a K, &'a V)> for RedBlackTree<K, V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(*k, *v);
        }
    }
}

impl<K: hash::Hash + Ord, V: hash::Hash> hash::Hash for RedBlackTree<K, V> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.iter().for_each(|e| e.hash(state));
    }
}

impl<K, Q, V> ops::Index<&'_ Q> for RedBlackTree<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, index: &'_ Q) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<K, Q, V> ops::IndexMut<&'_ Q> for RedBlackTree<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    fn index_mut(&mut self, index: &'_ Q) -> &mut Self::Output {
        self.get_mut(index).expect("no entry found for key")
    }
}

impl<K: Ord, V: PartialEq> PartialEq for RedBlackTree<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<K: Ord, V: Eq> Eq for RedBlackTree<K, V> {}

impl<K: Ord, V: PartialOrd> PartialOrd for RedBlackTree<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<K: Ord, V: Ord> Ord for RedBlackTree<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<K, V> RedBlackTree<K, V> {
    /// Creates an empty `RedBlackTree`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    ///
    /// map.insert(1, "a");
    /// ```
    pub const fn new() -> Self {
        Self {
            root: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    /// Removes all elements from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Returns whether the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub const fn len(&self) -> usize {
        self.len
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    /// Moves all elements from `other` into `Self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut a = RedBlackTree::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    ///
    /// let mut b = RedBlackTree::new();
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
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::<i32, &str>::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some((37, "b")));
    /// assert_eq!(map[&37], "c");
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        if self.is_empty() {
            self.root = Some(NodeRef::new(key, value));
            self.len += 1;
            return None;
        }
        match self.search_node(&key) {
            Ok(found) => {
                // replace
                let old_v = std::mem::replace(found.value_mut(), value);
                Some((key, old_v))
            }
            Err(target) => {
                let new_node = NodeRef::new(key, value);
                self.insert_node(new_node, target);
                self.len += 1;
                None
            }
        }
    }

    /// Removes a key from the map, returning the old value if the key was in.
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
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
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove_entry(&1), None);
    /// ```
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_empty() {
            return None;
        }
        let found = self.search_node(key).ok()?;
        self.len -= 1;
        Some(self.remove_node(found))
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
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
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_empty() {
            return None;
        }
        self.search_node(key).ok().map(|n| n.value_mut())
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_empty() {
            return None;
        }
        self.search_node(key).ok().map(|n| n.key_value())
    }

    /// Returns whether the map contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map = RedBlackTree::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
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
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map: RedBlackTree<i32, i32> = (0..8).map(|x| (x, x * 10)).collect();
    /// map.retain(|&k, _| k % 2 == 0);
    /// assert!(map.into_iter().eq(vec![(0, 0), (2, 20), (4, 40), (6, 60)]));
    /// ```
    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, mut f: F) {
        self.drain_filter(move |k, v| !f(k, v));
    }
}
