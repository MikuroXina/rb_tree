use crate::RbTreeSet;

use std::ops;

impl<T: Ord> Extend<T> for RbTreeSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for RbTreeSet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

impl<T: Ord + Clone> ops::Sub<&RbTreeSet<T>> for &RbTreeSet<T> {
    type Output = RbTreeSet<T>;

    fn sub(self, rhs: &RbTreeSet<T>) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> ops::BitXor<&RbTreeSet<T>> for &RbTreeSet<T> {
    type Output = RbTreeSet<T>;

    fn bitxor(self, rhs: &RbTreeSet<T>) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> ops::BitAnd<&RbTreeSet<T>> for &RbTreeSet<T> {
    type Output = RbTreeSet<T>;

    fn bitand(self, rhs: &RbTreeSet<T>) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Ord + Clone> ops::BitOr<&RbTreeSet<T>> for &RbTreeSet<T> {
    type Output = RbTreeSet<T>;

    fn bitor(self, rhs: &RbTreeSet<T>) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}
