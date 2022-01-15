use std::{borrow::Borrow, fmt, marker::PhantomData, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildIndex {
    Left,
    Right,
}

impl ChildIndex {
    pub fn is_left(self) -> bool {
        matches!(self, ChildIndex::Left)
    }

    pub fn is_right(self) -> bool {
        matches!(self, ChildIndex::Right)
    }
}

impl std::ops::Not for ChildIndex {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ChildIndex::Left => ChildIndex::Right,
            ChildIndex::Right => ChildIndex::Left,
        }
    }
}

pub struct Node<K, V> {
    parent: Option<NodeRef<K, V>>,
    #[allow(clippy::type_complexity)]
    children: (Option<NodeRef<K, V>>, Option<NodeRef<K, V>>),
    color: Color,
    key: K,
    value: V,
}

pub struct Root<K, V> {
    root: Option<NodeRef<K, V>>,
    len: usize,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> fmt::Debug for Root<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Root")
            .field("root", &self.root)
            .field("len", &self.len)
            .finish()
    }
}

impl<K, V> Default for Root<K, V> {
    fn default() -> Self {
        Self {
            root: None,
            len: 0,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> Root<K, V> {
    pub const fn new() -> Self {
        Self {
            root: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub const fn inner(&self) -> Option<NodeRef<K, V>> {
        self.root
    }

    #[allow(clippy::type_complexity)]
    pub fn search<Q>(&self, key: &Q) -> Option<Result<NodeRef<K, V>, (NodeRef<K, V>, ChildIndex)>>
    where
        K: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root.map(|r| r.search(key))
    }

    // Inserts a new node and returns Ok(the node inserted) or Err(old key-value entry).
    pub fn insert_node(&mut self, key: K, value: V) -> Result<NodeRef<K, V>, (K, V)>
    where
        K: Ord,
    {
        if self.is_empty() {
            let new_root = NodeRef::new(key, value);
            self.root = Some(new_root);
            self.len += 1;
            return Ok(new_root);
        }
        match self.root.unwrap().search(&key) {
            Ok(found) => {
                // only replace the value
                // Safety: The mutable reference is temporary.
                let old_v = std::mem::replace(unsafe { found.value_mut() }, value);
                Err((key, old_v))
            }
            Err((target, idx)) => {
                let new_node = NodeRef::new(key, value);
                debug_assert!(target.child(idx).is_none());

                unsafe {
                    target.set_child(idx, new_node);
                }

                new_node.balance_after_insert(&mut self.root);
                self.len += 1;
                Ok(new_node)
            }
        }
    }

    pub fn remove_node<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Ord + Borrow<Q>,
        Q: ?Sized + Ord,
    {
        let to_remove = self.root?.search(key).ok()?;

        self.len -= 1;

        if Some(to_remove) == self.root && to_remove.children() == (None, None) {
            // Safety: There is only `to_remove` in the tree, so just deallocate it.
            unsafe {
                self.root = None;
                return Some(to_remove.deallocate());
            }
        }
        // `to_remove` is not the root, has its parent.
        if let (Some(left), Some(right)) = to_remove.children() {
            // `to_remove` is needed to replace with the maximum node in the left.
            let max_in_left = left.max_child();
            let redundant = max_in_left.left();
            //  parent
            //    |
            //   to_remove
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
            //      to_remove
            //       /
            //     ...
            unsafe {
                let (idx, parent) = max_in_left.index_and_parent().unwrap();
                parent.set_child(idx, redundant);
                if let Some((idx, parent)) = to_remove.index_and_parent() {
                    parent.set_child(idx, max_in_left);
                } else {
                    self.root = Some(max_in_left);
                }
                max_in_left.set_child(ChildIndex::Left, left);
                max_in_left.set_child(ChildIndex::Right, right);
            }
        }

        if to_remove.is_red() {
            // Safety: If the node is red, it has no children. So it can be removed.
            unsafe {
                debug_assert!(to_remove.left().is_none());
                debug_assert!(to_remove.right().is_none());
                let (idx, parent) = to_remove.index_and_parent().unwrap();
                parent.clear_child(idx);
                return Some(to_remove.deallocate());
            }
        }

        // `to_remove` is black, has its parent, and has its one child at least.
        if let Some(red_child) = to_remove.left().or_else(|| to_remove.right()) {
            debug_assert!(red_child.is_red());
            debug_assert!(red_child.left().is_none());
            debug_assert!(red_child.right().is_none());
            // Safety: If `to_remove` has red child, the child can be colored as black and replaced with `to_remove`.
            //    parent
            //      |
            //    to_remove
            //      |
            // (red_child)
            // ↓
            //    parent
            //      |
            // [red_child]
            unsafe {
                if let Some((idx, parent)) = to_remove.index_and_parent() {
                    parent.set_child(idx, red_child);
                } else {
                    self.root = red_child.make_root();
                }
                red_child.set_color(Color::Black);
            }
        } else {
            // `to_remove` is not the root, black, and has no children.
            to_remove.balance_after_remove(&mut self.root);
        }

        // Safety: `to_remove` was removed from the tree.
        Some(unsafe { to_remove.deallocate() })
    }
}

pub struct NodeRef<K, V>(NonNull<Node<K, V>>);

impl<K, V> fmt::Debug for NodeRef<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeRef").field(&self.0).finish()
    }
}

impl<K, V> Clone for NodeRef<K, V> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<K, V> Copy for NodeRef<K, V> {}

impl<K, V> PartialEq for NodeRef<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K, V> Eq for NodeRef<K, V> {}

impl<K, V> NodeRef<K, V> {
    /// Constructs a new node of red-black tree with key and value. The node must be freed with [`deallocate`] after use.
    pub fn new(key: K, value: V) -> Self {
        let leaked = Box::leak(
            Node {
                parent: None,
                children: (None, None),
                color: Color::Red,
                key,
                value,
            }
            .into(),
        );
        NodeRef(leaked.into())
    }

    /// Deallocates the node and extract its key-value pair. You must not use the `NodeRef` after calling this method.
    ///
    /// # Safety
    ///
    /// This method must be called only once.
    pub unsafe fn deallocate(mut self) -> (K, V) {
        let this = self.0.as_mut();
        this.parent = None;
        this.children = (None, None);
        let this = Box::from_raw(self.0.as_ptr());
        (this.key, this.value)
    }

    /// Makes the node as root, has no parent.
    ///
    /// # Safety
    ///
    /// You must set the node into `root` of the tree.
    pub unsafe fn make_root(mut self) -> Option<Self> {
        self.0.as_mut().parent = None;
        Some(self)
    }

    /// Returns the borrowed key reference of the node.
    pub fn key<'a, Q>(self) -> &'a Q
    where
        K: Borrow<Q> + 'a,
        V: 'a,
        Q: ?Sized,
    {
        // Safety: The mutable reference of the key will not exist.
        unsafe { self.0.as_ref() }.key.borrow()
    }

    /// Returns the reference of key-value pair from the node.
    ///
    /// # Safety
    ///
    /// The mutable reference of its value must not exist.
    pub unsafe fn key_value<'a>(self) -> (&'a K, &'a V)
    where
        K: 'a,
        V: 'a,
    {
        let this = self.0.as_ref();
        (&this.key, &this.value)
    }

    /// Returns the mutable reference of key-value pair from the node. But the reference of key is shared because mutating the key breaks the invariants.
    ///
    /// # Safety
    ///
    /// The shared reference of its value must not exist.
    pub unsafe fn key_value_mut<'a>(mut self) -> (&'a K, &'a mut V)
    where
        K: 'a,
        V: 'a,
    {
        let this = self.0.as_mut();
        (&this.key, &mut this.value)
    }

    /// Returns the mutable reference of value pair from the node.
    ///
    /// # Safety
    ///
    /// The mutable reference of its value must not exist.
    pub unsafe fn value<'a>(self) -> &'a V
    where
        K: 'a,
    {
        &self.0.as_ref().value
    }

    /// Returns the mutable reference of value pair from the node.
    ///
    /// # Safety
    ///
    /// The shared reference of its value must not exist.
    pub unsafe fn value_mut<'a>(mut self) -> &'a mut V
    where
        K: 'a,
    {
        &mut self.0.as_mut().value
    }

    /// Returns whether the node colored as red.
    pub fn is_red(self) -> bool {
        // Safety: Only reading the color.
        unsafe { self.0.as_ref() }.color == Color::Red
    }

    /// Returns whether the node colored as black.
    pub fn is_black(self) -> bool {
        !self.is_red()
    }

    /// Returns the color of the node.
    pub fn color(self) -> Color {
        // Safety: Only reading the color.
        unsafe { self.0.as_ref() }.color
    }

    /// Colors the node with [`Color`].
    pub fn set_color(mut self, color: Color) {
        // Safety: Only writing the color.
        unsafe { self.0.as_mut() }.color = color;
    }

    /// Returns the parent node of the node.
    pub fn parent(self) -> Option<Self> {
        // Safety: Using the parent node will be guaranteed on caller.
        unsafe { self.0.as_ref() }.parent
    }

    /// Returns the grandparent node of the node.
    pub fn grandparent(self) -> Option<Self> {
        self.parent()?.parent()
    }

    /// Returns the uncle node of the node.
    pub fn uncle(self) -> Option<Self> {
        self.parent()?.sibling()
    }

    /// Returns the sibling node of the node.
    pub fn sibling(self) -> Option<Self> {
        let index = self.index_on_parent()?;
        let parent = self.parent()?;
        parent.child(!index)
    }

    /// Returns the close nephew node of the node.
    pub fn close_nephew(self) -> Option<Self> {
        let index = self.index_on_parent()?;
        let sibling = self.sibling()?;
        sibling.child(index)
    }

    /// Returns the distant nephew node of the node.
    pub fn distant_nephew(self) -> Option<Self> {
        let index = self.index_on_parent()?;
        let sibling = self.sibling()?;
        sibling.child(!index)
    }

    /// Returns the children of the node.
    pub fn children(self) -> (Option<Self>, Option<Self>) {
        // Safety: Using the children node will be guaranteed on caller.
        let this = unsafe { self.0.as_ref() };
        this.children
    }

    /// Returns the child of the node.
    pub fn child(self, idx: ChildIndex) -> Option<Self> {
        // Safety: Using the child node will be guaranteed on caller.
        let this = unsafe { self.0.as_ref() };
        match idx {
            ChildIndex::Left => this.children.0,
            ChildIndex::Right => this.children.1,
        }
    }

    /// Returns the left child of the node.
    pub fn left(self) -> Option<Self> {
        self.child(ChildIndex::Left)
    }

    /// Returns the right child of the node.
    pub fn right(self) -> Option<Self> {
        self.child(ChildIndex::Right)
    }

    /// Clears the child link on `idx` edge. The removed child node must be re-connected to another node with [`set_child`] or deallocated.
    ///
    /// # Safety
    ///
    /// The child edge on `idx` must be occupied.
    pub unsafe fn clear_child(mut self, idx: ChildIndex) -> Self {
        let this = self.0.as_mut();
        if let Some(mut child) = self.child(idx) {
            child.0.as_mut().parent = None;
        }
        let child = match idx {
            ChildIndex::Left => &mut this.children.0,
            ChildIndex::Right => &mut this.children.1,
        };
        debug_assert!(child.is_some(), "the child on {:?} must be occupied", idx);
        child.take().unwrap()
    }

    /// Make a child link to `new_child` on `idx` edge. And returns the old child entry.
    pub unsafe fn set_child(
        mut self,
        idx: ChildIndex,
        new_child: impl Into<Option<Self>>,
    ) -> Option<Self> {
        let new_child = new_child.into();
        debug_assert_ne!(Some(self), new_child);
        let this = self.0.as_mut();
        if let Some(mut new_child) = new_child {
            new_child.0.as_mut().parent = Some(self);
        }
        match idx {
            ChildIndex::Left => std::mem::replace(&mut this.children.0, new_child),
            ChildIndex::Right => std::mem::replace(&mut this.children.1, new_child),
        }
    }

    /// Returns where the node is on its parent.
    pub fn index_on_parent(self) -> Option<ChildIndex> {
        let parent = self.parent()?;
        let child = parent.left();
        Some(if Some(self) == child {
            ChildIndex::Left
        } else {
            ChildIndex::Right
        })
    }

    /// Returns the parent node and where the node is on its parent.
    pub fn index_and_parent(self) -> Option<(ChildIndex, Self)> {
        self.index_on_parent().zip(self.parent())
    }

    pub fn search<Q>(mut self, key: &Q) -> Result<Self, (Self, ChildIndex)>
    where
        K: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        loop {
            let idx = match key.cmp(self.key()) {
                std::cmp::Ordering::Less => ChildIndex::Left,
                std::cmp::Ordering::Equal => return Ok(self),
                std::cmp::Ordering::Greater => ChildIndex::Right,
            };
            self = self.child(idx).ok_or((self, idx))?;
        }
    }

    pub fn min_child(self) -> NodeRef<K, V> {
        let mut current = self;
        while let Some(left) = current.left() {
            current = left;
        }
        current
    }

    pub fn max_child(self) -> NodeRef<K, V> {
        let mut current = self;
        while let Some(right) = current.right() {
            current = right;
        }
        current
    }
}
