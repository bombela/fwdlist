use super::ListIterMut;
use crate::{Cursor, List, Node};
use std::mem;

/// Extra operations on mutable iterator - **Unstable API**.
impl<'a, T> ListIterMut<'a, T> {
    /// Returns a reference to the next element, without moving the iterator.
    pub fn peek_next(&self) -> Option<&T> {
        self.next_link.as_ref().map(|node| &node.value)
    }

    /// Returns a mutable reference to the next element, without moving the
    /// iterator.
    pub fn peek_next_mut(&mut self) -> Option<&mut T> {
        self.next_link.as_mut().map(|node| &mut node.value)
    }

    /// Insert `v` just after the element most recently returned by `.next()` in
    /// O(1).
    ///
    /// The inserted element does not appear in the iteration.
    pub fn insert_next(&mut self, v: T) {
        let new_node = Node::new_boxed(v, self.next_link.take());
        *self.next_link = Some(new_node);
        *self.list_len += 1;
        self.len += 1;
        self.next();
    }

    /// Remove the element after the one most recently returned by `.next()` in
    /// O(1);
    ///
    /// Returns the removed value or None if the iterator is already at the end
    /// of the list.
    pub fn remove_next(&mut self) -> Option<T> {
        self.next_link.take().map(|node| {
            let (value, next) = node.take();
            *self.next_link = next;
            *self.list_len -= 1;
            self.len -= 1;
            value
        })
    }

    /// Truncate the list right after the element most recently return by
    /// `.next()` in O(1).
    ///
    /// * returns a new list owning all the elements after the one most recently
    /// returned by `.next()`.
    /// * the iterator is now exhausted since the list got truncated.
    /// * returns an empty list if the iterator was already exhausted.
    pub fn truncate_next(&mut self) -> List<T> {
        let tail_link = self.next_link.take();
        *self.list_len -= self.len;
        List {
            len: mem::replace(&mut self.len, 0),
            head: tail_link,
        }
    }
}

/// Convert the mutable iterator into a cursor **unstable* API*.
impl<'a, T> Into<Cursor<'a, T>> for ListIterMut<'a, T> {
    fn into(self) -> Cursor<'a, T> {
        Cursor {
            position: *self.list_len - self.len,
            list_len: self.list_len,
            next_link: self.next_link,
        }
    }
}

#[test]
fn mutref_iter_advanced() {
    let mut l = (0..10).collect::<List<_>>();
    assert_eq!(l.len(), 10);
    {
        let mut iter = l.iter_mut();
        for i in 0..9 {
            let v = *iter.next().unwrap();
            if i == 6 {
                assert_eq!(v, 150);
            } else if i > 7 {
                assert_eq!(v, i + 1);
            } else {
                assert_eq!(v, i);
            }
            if i == 3 {
                iter.insert_next(42);
            }
            if i < 8 {
                assert_eq!(*iter.peek_next().unwrap(), i + 1);
            }
            if i == 5 {
                *iter.peek_next_mut().unwrap() = 150;
            }
            if i == 7 {
                assert_eq!(iter.remove_next(), Some(8));
            }
        }
    }
    assert_eq!(l.len(), 10);

    let mut iter = l.into_iter();
    for &i in &[0, 1, 2, 3, 42, 4, 5, 150, 7, 9] {
        assert_eq!(iter.next(), Some(i));
    }
}
