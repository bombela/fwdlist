use crate::List;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

/// Drop the list in O(n).
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// A default empty list.
impl<T> Default for List<T> {
    fn default() -> List<T> {
        List::new()
    }
}

/// A debug formatter.
impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter()
            .fold(f.debug_list(), |mut b, e| {
                b.entry(e);
                b
            })
            .finish()
    }
}

/// Clone a list in O(n).
///
/// `clone_from()` will reuse as many nodes from `self` as possible to avoid
/// reallocation.
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> List<T> {
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, source: &Self) {
        let mut src_iter = source.iter().cloned();
        let mut dst_iter = self.iter_mut();
        while let Some(dst_v) = dst_iter.next() {
            if let Some(v) = src_iter.next() {
                *dst_v = v;
            } else {
                dst_iter.truncate_next();
                return;
            }
        }
        for src_v in src_iter {
            dst_iter.insert_next(src_v);
        }
    }
}

/// Construct a list from the content of the iterator `iter` in O(n).
impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> List<T> {
        let mut r = List::new();
        r.extend(iter);
        r
    }
}

/// Extend the list from the content of `iter` in O(n).
impl<T> Extend<T> for List<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut tail = self.iter_mut();
        for _ in &mut tail {}
        for v in iter {
            tail.insert_next(v)
        }
    }
}

impl<T: Hash> Hash for List<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for v in self {
            v.hash(state);
        }
    }
}
