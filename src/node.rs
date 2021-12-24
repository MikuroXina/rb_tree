use crate::Ptr;

use std::ptr::NonNull;

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
    parent: Ptr<Node<K, V>>,
    #[allow(clippy::type_complexity)]
    children: (Ptr<Node<K, V>>, Ptr<Node<K, V>>),
    color: Color,
    key: K,
    value: V,
}

impl<K, V> Node<K, V> {
    pub fn new(parent: Option<NonNull<Self>>, key: K, value: V) -> Self {
        Self {
            parent,
            children: (None, None),
            color: Color::Red,
            key,
            value,
        }
    }

    pub fn is_red(this: NonNull<Self>) -> bool {
        unsafe { this.as_ref() }.color == Color::Red
    }

    pub fn is_black(this: NonNull<Self>) -> bool {
        !Self::is_red(this)
    }

    pub fn color(this: NonNull<Self>) -> Color {
        unsafe { this.as_ref() }.color
    }

    pub fn set_color(mut this: NonNull<Self>, color: Color) {
        unsafe { this.as_mut() }.color = color;
    }

    pub fn parent(this: NonNull<Self>) -> Ptr<Self> {
        unsafe { this.as_ref() }.parent
    }

    pub fn grandparent(this: NonNull<Self>) -> Ptr<Self> {
        let parent = Self::parent(this)?;
        unsafe { parent.as_ref() }.parent
    }

    pub fn uncle(this: NonNull<Self>) -> Ptr<Self> {
        let parent = Self::parent(this)?;
        let index = Self::index_on_parent(parent)?;
        let grandparent = Self::grandparent(this)?;
        Self::child(grandparent, !index)
    }

    pub fn sibling(this: NonNull<Self>) -> Ptr<Self> {
        let index = Self::index_on_parent(this)?;
        let parent = Self::parent(this)?;
        Self::child(parent, !index)
    }

    pub fn close_nephew(this: NonNull<Self>) -> Ptr<Self> {
        let index = Self::index_on_parent(this)?;
        let sibling = Self::sibling(this)?;
        Self::child(sibling, index)
    }
    pub fn distant_nephew(this: NonNull<Self>) -> Ptr<Self> {
        let index = Self::index_on_parent(this)?;
        let sibling = Self::sibling(this)?;
        Self::child(sibling, !index)
    }

    pub fn child(this: NonNull<Self>, idx: ChildIndex) -> Ptr<Self> {
        let children = unsafe { this.as_ref() }.children;
        match idx {
            ChildIndex::Left => children.0,
            ChildIndex::Right => children.1,
        }
    }

    pub fn set_child(mut this: NonNull<Self>, idx: ChildIndex, new_child: Ptr<Self>) {
        if let Some(mut child) = new_child {
            unsafe { child.as_mut() }.parent = Some(this);
        }
        let this = unsafe { this.as_mut() };
        match idx {
            ChildIndex::Left => this.children.0 = new_child,
            ChildIndex::Right => this.children.1 = new_child,
        }
    }

    pub fn into_element(self) -> (K, V) {
        (self.key, self.value)
    }

    pub fn index_on_parent(this: NonNull<Self>) -> Option<ChildIndex> {
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
    pub fn which_to_insert(new_node: NonNull<Self>, target: NonNull<Self>) -> ChildIndex {
        if unsafe { new_node.as_ref() }.key < unsafe { target.as_ref() }.key {
            ChildIndex::Left
        } else {
            ChildIndex::Right
        }
    }
}
