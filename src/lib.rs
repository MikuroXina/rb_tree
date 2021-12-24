use std::{marker::PhantomData, ptr::NonNull};

type Ptr<T> = Option<NonNull<T>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildIndex {
    Left,
    Right,
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

struct Node<K, V> {
    parent: Ptr<Node<K, V>>,
    children: (Ptr<Node<K, V>>, Ptr<Node<K, V>>),
    color: Color,
    key: K,
    value: V,
}

impl<K, V> Node<K, V> {
    fn new(parent: Option<NonNull<Self>>, color: Color, key: K, value: V) -> Self {
        Self {
            parent,
            children: (None, None),
            color,
            key,
            value,
        }
    }

    fn is_red(this: NonNull<Self>) -> bool {
        unsafe { this.as_ref() }.color == Color::Red
    }

    fn is_black(this: NonNull<Self>) -> bool {
        !Self::is_red(this)
    }

    fn color(this: NonNull<Self>) -> Color {
        unsafe { this.as_ref() }.color
    }

    fn set_color(mut this: NonNull<Self>, color: Color) {
        unsafe { this.as_mut() }.color = color;
    }

    fn parent(this: NonNull<Self>) -> Ptr<Self> {
        unsafe { this.as_ref() }.parent
    }

    fn grandparent(this: NonNull<Self>) -> Ptr<Self> {
        let parent = Self::parent(this)?;
        unsafe { parent.as_ref() }.parent
    }

    fn uncle(this: NonNull<Self>) -> Ptr<Self> {
        let parent = Self::parent(this)?;
        let index = Self::index_on_parent(parent)?;
        let grandparent = Self::grandparent(this)?;
        Self::child(grandparent, !index)
    }

    fn sibling(this: NonNull<Self>) -> Ptr<Self> {
        let index = Self::index_on_parent(this)?;
        let parent = Self::parent(this)?;
        Self::child(parent, !index)
    }

    fn close_nephew(this: NonNull<Self>) -> Ptr<Self> {
        let index = Self::index_on_parent(this)?;
        let sibling = Self::sibling(this)?;
        Self::child(sibling, index)
    }

    fn distant_nephew(this: NonNull<Self>) -> Ptr<Self> {
        let index = Self::index_on_parent(this)?;
        let sibling = Self::sibling(this)?;
        Self::child(sibling, !index)
    }

    fn child(this: NonNull<Self>, idx: ChildIndex) -> Ptr<Self> {
        let children = unsafe { this.as_ref() }.children;
        match idx {
            ChildIndex::Left => children.0,
            ChildIndex::Right => children.1,
        }
    }

    fn set_child(mut this: NonNull<Self>, idx: ChildIndex, new_child: Ptr<Self>) {
        if let Some(mut child) = new_child {
            unsafe { child.as_mut() }.parent = Some(this);
        }
        let this = unsafe { this.as_mut() };
        match idx {
            ChildIndex::Left => this.children.0 = new_child,
            ChildIndex::Right => this.children.1 = new_child,
        }
    }

    fn into_element(self) -> (K, V) {
        (self.key, self.value)
    }

    fn index_on_parent(this: NonNull<Self>) -> Option<ChildIndex> {
        let parent = Self::parent(this)?;
        let child = unsafe { parent.as_ref() }.children.0?;
        Some(if child == this {
            ChildIndex::Left
        } else {
            ChildIndex::Right
        })
    }
}

impl<K: Ord, V> Node<K, V> {
    fn which_to_insert(new_node: NonNull<Self>, target: NonNull<Self>) -> ChildIndex {
        if unsafe { new_node.as_ref() }.key < unsafe { target.as_ref() }.key {
            ChildIndex::Left
        } else {
            ChildIndex::Right
        }
    }
}

pub struct RedBlackTree<K, V> {
    root: Ptr<Node<K, V>>,
    len: usize,
    _phantom: PhantomData<Box<Node<K, V>>>,
}

// private methods
impl<K: Ord, V> RedBlackTree<K, V> {
    fn rotate(&mut self, node: NonNull<Node<K, V>>, pivot_idx: ChildIndex) -> NonNull<Node<K, V>> {
        //           [node]
        //            /   \
        //        [pivot] [be_fallen]
        //         /   \
        // [be_risen] [be_moved]
        //            ↓
        //        [pivot]
        //         /   \
        // [be_risen] [node]
        //             /   \
        //     [be_moved] [be_fallen]
        let parent = Node::parent(node);
        let pivot = Node::child(node, pivot_idx).expect("pivot must be found");
        let be_moved = Node::child(pivot, !pivot_idx);

        match Node::index_on_parent(node) {
            // update `parent`'s child
            Some(idx) => {
                Node::set_child(parent.unwrap(), idx, Some(pivot));
            }
            None => {
                self.root = Some(pivot);
            }
        }
        // update `pivot`'s child
        Node::set_child(pivot, !pivot_idx, Some(node));
        // update `node`'s child
        Node::set_child(node, pivot_idx, be_moved);
        pivot
    }

    fn insert_node(&mut self, mut new_node: Box<Node<K, V>>, target: Ptr<Node<K, V>>) {
        new_node.color = Color::Red;
        let ptr = NonNull::new(Box::into_raw(new_node)).unwrap();
        if target.is_none() {
            self.root = Some(ptr);
            return;
        }
        let target = target.unwrap();
        let idx = Node::which_to_insert(ptr, target);
        Node::set_child(target, idx, Some(ptr));
        let mut new_node = ptr;

        // re-balance
        while let Some(parent) = Node::parent(new_node) {
            if Node::is_black(parent) {
                // if the parent is black, the tree is well balanced.
                return;
            }
            // the parent is red
            if Node::parent(parent).is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                Node::set_color(parent, Color::Black);
                return;
            }
            // the parent is red and the grandparent exists
            if Node::uncle(new_node).map(Node::is_red).unwrap_or(false) {
                // if the parent and the uncle is red, they will be black and the grandparent will be red.
                Node::set_color(parent, Color::Black);
                Node::set_color(Node::uncle(new_node).unwrap(), Color::Black);
                let grandparent = Node::grandparent(new_node).unwrap();
                Node::set_color(grandparent, Color::Red);
                // then nodes above the grandparent needs to re-balance.
                new_node = grandparent;
                continue;
            }
            // the parent is red and the uncle is black
            if Node::index_on_parent(parent) != Node::index_on_parent(new_node) {
                // if the nodes are connected:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (parent) [uncle] | [uncle] (parent)
                //      \           |           /
                //    (new_node)    |      (new_node)
                self.rotate(parent, Node::index_on_parent(new_node).unwrap());
                // then rotated:
                //   [grandparent]    |  [grandparent]
                //     /     \        |     /     \
                // (new_node) [uncle] | [uncle] (new_node)
                //   /                |             \
                // (parent)           |          (parent)
                new_node = parent;
            }
            // the nodes are connected:
            //   [grandparent]  |  [grandparent]
            //     /     \      |     /     \
            // (parent) [uncle] | [uncle] (parent)
            //   /              |             \
            // (new_node)       |          (new_node)
            Node::set_color(parent, Color::Black);
            Node::set_color(Node::grandparent(new_node).unwrap(), Color::Red);
            // then colored:
            //   (grandparent)  |  (grandparent)
            //     /     \      |     /     \
            // [parent] [uncle] | [uncle] [parent]
            //   /              |             \
            // (new_node)       |          (new_node)
            self.rotate(
                Node::grandparent(new_node).unwrap(),
                Node::index_on_parent(new_node).unwrap(),
            );
            // finished:
            //       [parent]           |          [parent]
            //        /    \            |           /    \
            // (new_node) (grandparent) | (grandparent) (new_node)
            //                \         |      /
            //              [uncle]     |   [uncle]
            return;
        }
    }
}

impl<K, V> Drop for RedBlackTree<K, V> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<K, V> RedBlackTree<K, V> {
    pub const fn new() -> Self {
        Self {
            root: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!()
    }
}
