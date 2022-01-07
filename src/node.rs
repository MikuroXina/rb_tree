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
        let ptr = Box::into_raw(
            Node {
                parent: None,
                children: (None, None),
                color: Color::Red,
                key,
                value,
            }
            .into(),
        );
        NodeRef(NonNull::new(ptr).unwrap())
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

    /// Clears the child link on `idx` edge. The removed child node must be re-connected to another node with [`set_child`].
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

    /// Make a child link to `new_child` on `idx` edge.
    ///
    /// # Safety
    ///
    /// The child on `idx` must be empty before calling this.
    pub unsafe fn set_child(mut self, idx: ChildIndex, new_child: impl Into<Option<Self>>) {
        let new_child = new_child.into();
        debug_assert_ne!(Some(self), new_child);
        let this = self.0.as_mut();
        if let Some(mut new_child) = new_child {
            new_child.0.as_mut().parent = Some(self);
        }
        match idx {
            ChildIndex::Left => this.children.0 = new_child,
            ChildIndex::Right => this.children.1 = new_child,
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
}

impl<K: Ord, V> NodeRef<K, V> {
    /// Searches the nearest node of `key`, or position the node to be inserted.
    pub fn search<Q>(mut self, key: &Q) -> Result<Self, (Self, ChildIndex)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        loop {
            let idx = match key.cmp(self.key()) {
                std::cmp::Ordering::Less => ChildIndex::Left,
                std::cmp::Ordering::Equal => return Ok(self),
                std::cmp::Ordering::Greater => ChildIndex::Right,
            };
            self = self.child(idx).ok_or((self, idx))?
        }
    }
}
