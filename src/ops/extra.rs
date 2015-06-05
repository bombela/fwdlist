use std::{mem};
use ::{List, Node};

/// Extra operations on the list - **Unstable API**.
impl<T> List<T> {
    /// Moves all elements from `other` to the end of the list in O(self.len());
    pub fn append(&mut self, other: &mut List<T>) {
        *self.last_link() = other.head.take();
        self.len += mem::replace(&mut other.len, 0);
    }

    /// Splits the list into two at the given index in O(at).
    ///
    /// * Returns everything after the given index, including the index.
    /// * if `at == self.len()`, returns an empty list in O(1).
    /// * Panics if `at > self.len()`.
    #[inline(never)]
    pub fn split_off(&mut self, at: usize) -> List<T> {
        assert!(at <= self.len, "Cannot split off at a nonexistent index");
        if at == self.len { return List::new(); }

        let tail_link;
        let mut head_link = &mut self.head;
        let mut i = 0;
        loop {
            if i == at {
                tail_link = head_link.take();
                break
            }
            if let Some(ref mut node) = *{head_link} {
                let Node(_, ref mut next_link) = **node;
                head_link = next_link;
                i += 1;
            } else {
                unreachable!();
            }
        }
        List{
            len: mem::replace(&mut self.len, at) - at,
            head: tail_link,
        }
    }
}

#[test]
fn append() {
    let mut a = List::new();
    let mut b = List::new();
    for i in 0..5 {
        b.push_front(i);
    }
    assert_eq!(a.len(), 0);
    assert_eq!(b.len(), 5);
    a.append(&mut b);
    assert_eq!(a.len(), 5);
    assert_eq!(b.len(), 0);
}

#[test]
fn split_off() {
    let mut a = List::new();
    for i in 0..20 { a.push_front(i); }
    let b = a.split_off(7);
    assert_eq!(a.len(), 7);
    assert_eq!(b.len(), 13);
    assert_eq!(*a.front().unwrap(), 19);
    assert_eq!(*a.back().unwrap(), 13);
    assert_eq!(*b.front().unwrap(), 12);
    assert_eq!(*b.back().unwrap(), 0);

    let mut a = List::new();
    for i in 0..10 { a.push_front(i); }
    let b = a.split_off(10);
    assert_eq!(a.len(), 10);
    assert_eq!(b.len(), 0);
}

#[test] #[should_panic]
fn split_off_panic() {
    let mut a = List::new();
    for i in 0..10 { a.push_front(i); }
    let _ = a.split_off(11);
}
