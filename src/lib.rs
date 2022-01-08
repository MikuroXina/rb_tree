#![deny(clippy::undocumented_unsafe_blocks)]

mod balance;
pub mod entry;
pub mod iter;
mod mem;
mod node;
#[cfg(test)]
mod tests;

use node::{ChildIndex, Node, NodeRef};

use std::{borrow::Borrow, fmt, hash, marker::PhantomData, ops};

use crate::node::Color;

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
        debug_assert!(target.child(idx).is_none());

        // Safety: the child entry of the target is empty.
        unsafe { target.set_child(idx, new_node) };

        self.balance_after_insert(new_node);
    }

    fn remove_node(&mut self, mut node: NodeRef<K, V>) -> (K, V) {
        if self.len == 0 {
            // Safety: There is only `node` in the tree, so just deallocate it.
            unsafe { return node.deallocate() }
        }
        // `node` is not the root, has its parent.
        if let (Some(left), Some(right)) = node.children() {
            // `node` is needed to replace with the maximum node in the left.
            let mut max_in_left = left;
            while let Some(max) = max_in_left.right() {
                max_in_left = max;
            }
            let max_in_left = max_in_left;
            // Safety: The color, parent and children of `node` is replaced with `max_in_left`. Then `node` has only one child.
            //  parent
            //    |
            //   node
            //   /  \
            // left right
            // /  \
            //    ...
            //      \
            //   max_in_left
            //      /
            //    ...
            // ↓
            //   parent
            //     |
            // max_in_left
            //    /  \
            //  left right
            //  /  \
            //     ...
            //       \
            //      node
            //       /
            //     ...
            unsafe {
                let node_color = node.color();
                node.clear_child(ChildIndex::Right);
                node.set_child(ChildIndex::Left, max_in_left.left());
                node.set_color(max_in_left.color());
                max_in_left.set_child(ChildIndex::Left, left);
                max_in_left.set_child(ChildIndex::Right, right);
                max_in_left.set_color(node_color);
                node = max_in_left;
            }
        }

        if let Some(child) = node.left().xor(node.right()) {
            // If `node` has one child, the color of the child must be red.
            debug_assert!(child.is_red());
        }

        if node.is_red() {
            // Safety: If the node is red, it must have no children. So it can be removed.
            unsafe {
                debug_assert!(node.left().is_none());
                debug_assert!(node.right().is_none());
                let (idx, parent) = node.index_and_parent().unwrap();
                parent.clear_child(idx);
                return node.deallocate();
            }
        }

        // `node` is black, has its parent, and has its one child at least.
        if let Some(red_child) = node
            .left()
            .or_else(|| node.right())
            .filter(|child| child.is_red())
        {
            // Safety: If `node` has red child, the child can be colored as red and replaced with `node`.
            unsafe {
                let red_child_idx = red_child.index_on_parent().unwrap();
                node.clear_child(red_child_idx);

                let (idx, parent) = node.index_and_parent().unwrap();
                parent.set_child(idx, red_child);
                red_child.set_color(Color::Black);
                red_child.set_child(red_child_idx, node);
                node = red_child;
            }
        }

        // `node` is not the root, black, and has no children.
        self.balance_after_remove(node);

        // Safety: `node` has no children, so it can be removed.
        unsafe {
            let (idx, parent) = node.index_and_parent().unwrap();
            parent.clear_child(idx);
            node.deallocate()
        }
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
        // Safety: `self` will not be used after.
        unsafe { drop(std::ptr::read(self).into_iter()) }
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
            self.len += 1;
            self.root = Some(NodeRef::new(key, value));
            return None;
        }
        match self.search_node(&key) {
            Ok(found) => {
                // replace
                // Safety: The mutable reference is temporary.
                let old_v = std::mem::replace(unsafe { found.value_mut() }, value);
                Some((key, old_v))
            }
            Err(target) => {
                let new_node = NodeRef::new(key, value);
                self.len += 1;
                self.insert_node(new_node, target);
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
        // Safety: The mutable reference will not live longer than `&mut self`.
        self.search_node(key).ok().map(|n| unsafe { n.value_mut() })
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
        // Safety: The reference of key-value pair will not live longer than `&self`.
        self.search_node(key).ok().map(|n| unsafe { n.key_value() })
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
