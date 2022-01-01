use crate::RedBlackTree;

impl<K: Ord, V> RedBlackTree<K, V> {
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        Entry { key, tree: self }
    }
}

pub struct Entry<'a, K, V> {
    key: K,
    tree: &'a mut RedBlackTree<K, V>,
}

impl<'a, K: Ord, V> Entry<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        if let Ok(found) = self.tree.search_node(&self.key) {
            found.insert(default)
        } else {
            self.tree.get_mut(&self.key).unwrap()
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        self.or_insert_with_key(move |_| default())
    }

    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        if let Ok(found) = self.tree.search_node(&self.key) {
            found.insert(default(&self.key))
        } else {
            self.tree.get_mut(&self.key).unwrap()
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        if let Some(entry) = self.tree.get_mut(&self.key) {
            f(entry);
        }
        self
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }
}
