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
        let index = Self::index_on_parent(this)?;
        let parent = Self::parent(this).unwrap();
        Self::child(parent, !index)
    }

    fn child(this: NonNull<Self>, idx: ChildIndex) -> Ptr<Self> {
        let children = unsafe { this.as_ref() }.children;
        match idx {
            ChildIndex::Left => children.0,
            ChildIndex::Right => children.1,
        }
    }

    fn child_mut(&mut self, idx: ChildIndex) -> &mut Ptr<Self> {
        match idx {
            ChildIndex::Left => &mut self.children.0,
            ChildIndex::Right => &mut self.children.1,
        }
    }

    fn into_element(self: Box<Self>) -> (K, V) {
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

pub struct RedBlackTree<K, V> {
    root: Ptr<Node<K, V>>,
    len: usize,
    _phantom: PhantomData<Box<Node<K, V>>>,
}

// private methods
impl<K: Ord, V> RedBlackTree<K, V> {
    fn rotate(
        &mut self,
        mut node: NonNull<Node<K, V>>,
        pivot_idx: ChildIndex,
    ) -> NonNull<Node<K, V>> {
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
        let mut pivot = Node::child(node, pivot_idx).expect("pivot must be found");
        let be_moved = Node::child(pivot, !pivot_idx);

        if let Some(mut be_moved) = be_moved {
            // update `be_moved`'s parent
            let be_moved = unsafe { be_moved.as_mut() };
            be_moved.parent = Some(node);
        }
        {
            // update `node`'s child and parent
            let node = unsafe { node.as_mut() };
            *node.child_mut(pivot_idx) = be_moved;
            node.parent = Some(pivot);
        }
        {
            // update `pivot`'s child and parent
            let pivot = unsafe { pivot.as_mut() };
            *pivot.child_mut(!pivot_idx) = Some(node);
            pivot.parent = parent;
        }
        match Node::index_on_parent(node) {
            // update `parent`'s child
            Some(idx) => {
                let mut parent = parent.unwrap();
                *unsafe { parent.as_mut() }.child_mut(idx) = Some(pivot);
            }
            None => {
                self.root = Some(pivot);
            }
        }
        pivot
    }

    fn insert_node(&mut self, mut new_node: Box<Node<K, V>>, target: Ptr<Node<K, V>>) {
        if target.is_none() {
            self.root = NonNull::new(Box::into_raw(new_node));
            return;
        }
        new_node.color = Color::Red;
        new_node.parent = target;
        let mut new_node = {
            let mut target = target.unwrap();
            let idx = if new_node.key < unsafe { target.as_ref() }.key {
                ChildIndex::Left
            } else {
                ChildIndex::Right
            };
            let new_node = NonNull::new(Box::into_raw(new_node));
            {
                let target = unsafe { target.as_mut() };
                *target.child_mut(idx) = new_node;
            }
            new_node.unwrap()
        };

        // re-balance
        enum Case {
            ParentIsBlack,
            NoGrandparent,
            UncleIsRed,
            InnerGrandchild,
            OuterGrandchild,
        }
        let parent = Node::parent(new_node).unwrap();
        let mut state = if Node::is_black(parent) {
            Case::ParentIsBlack
        } else if Node::parent(parent).is_none() {
            Case::NoGrandparent
        } else if Node::uncle(new_node).map(Node::is_red).unwrap_or(false) {
            Case::UncleIsRed
        } else if Node::index_on_parent(parent) != Node::index_on_parent(new_node) {
            Case::InnerGrandchild
        } else {
            Case::OuterGrandchild
        };
        while let Some(parent) = Node::parent(new_node) {
            match state {
                Case::ParentIsBlack => return,
                Case::NoGrandparent => {
                    Node::set_color(parent, Color::Black);
                    return;
                }
                Case::UncleIsRed => {
                    Node::set_color(parent, Color::Black);
                    Node::set_color(Node::uncle(new_node).unwrap(), Color::Black);
                    let grandparent = Node::grandparent(new_node).unwrap();
                    Node::set_color(grandparent, Color::Red);
                    new_node = grandparent;
                }
                Case::InnerGrandchild => {
                    self.rotate(parent, Node::index_on_parent(new_node).unwrap());
                    new_node = parent;
                    state = Case::OuterGrandchild;
                }
                Case::OuterGrandchild => {
                    self.rotate(
                        Node::grandparent(new_node).unwrap(),
                        Node::index_on_parent(new_node).unwrap(),
                    );
                    Node::set_color(parent, Color::Black);
                    Node::set_color(Node::grandparent(new_node).unwrap(), Color::Red);
                    return;
                }
            }
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
