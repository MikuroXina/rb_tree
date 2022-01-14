use std::{borrow::Borrow, fmt, ptr::NonNull};

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
        if let Some((idx, parent)) = self.index_and_parent() {
            parent.clear_child(idx);
            this.parent = None;
        }
        if let Some(mut left) = this.children.0.take() {
            left.0.as_mut().parent = None;
        }
        if let Some(mut right) = this.children.1.take() {
            right.0.as_mut().parent = None;
        }
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

    pub fn insert_node(self, (target, idx): (Self, ChildIndex), root: &mut Option<Self>) {
        debug_assert!(target.child(idx).is_none());

        unsafe {
            target.set_child(idx, self);
        }

        self.balance_after_insert(root);
    }

    pub fn remove_node(self, root: &mut Option<Self>) -> (K, V) {
        if self.parent().is_none() {
            // Safety: There is only `node` in the tree, so just deallocate it.
            unsafe {
                *root = None;
                return self.deallocate();
            }
        }
        // `node` is not the root, has its parent.
        if let (Some(left), Some(right)) = self.children() {
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
                let (idx, parent) = self.index_and_parent().unwrap();
                let node_color = self.color();
                self.set_child(ChildIndex::Right, None);
                self.set_child(ChildIndex::Left, max_in_left.left());
                self.set_color(max_in_left.color());
                max_in_left.set_child(ChildIndex::Left, left);
                max_in_left.set_child(ChildIndex::Right, right);
                max_in_left.set_color(node_color);
                parent.set_child(idx, max_in_left);
            }
        }

        if self.is_red() {
            // Safety: If the node is red, it has no children. So it can be removed.
            unsafe {
                debug_assert!(self.left().is_none());
                debug_assert!(self.right().is_none());
                let (idx, parent) = self.index_and_parent().unwrap();
                parent.clear_child(idx);
                return self.deallocate();
            }
        }

        // `node` is black, has its parent, and has its one child at least.
        if let Some(red_child) = self.left().or_else(|| self.right()) {
            debug_assert!(red_child.is_red());
            debug_assert!(red_child.left().is_none());
            debug_assert!(red_child.right().is_none());
            // Safety: If `node` has red child, the child can be colored as black and replaced with `node`.
            //    parent
            //      |
            //    node
            //      |
            // (red_child)
            // ↓
            //    parent
            //      |
            // [red_child]
            unsafe {
                if let Some((idx, parent)) = self.index_and_parent() {
                    parent.set_child(idx, red_child);
                } else {
                    *root = red_child.make_root();
                }
                red_child.set_color(Color::Black);
            }
        } else {
            // `node` is not the root, black, and has no children.
            self.balance_after_remove(root);
        }

        // Safety: `node` was removed from the tree.
        unsafe { self.deallocate() }
    }

    pub fn first_node(self) -> Option<NodeRef<K, V>> {
        let mut current = self;
        while let Some(left) = current.left() {
            current = left;
        }
        Some(current)
    }

    pub fn last_node(self) -> Option<NodeRef<K, V>> {
        let mut current = self;
        while let Some(right) = current.right() {
            current = right;
        }
        Some(current)
    }
}
