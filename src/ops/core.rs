use ::{List, Node};

impl<T> List<T> {

    /// A new empty list.
    pub fn new() -> List<T> {
        List{len: 0, head: None}
    }

    /// The size of the list in O(1).
    pub fn len(&self) -> usize { self.len }

    /// Returns true if list is empty in O(1);
    pub fn is_empty(&self) -> bool { self.head.is_none() }

    /// Push a new element at the front of the list in O(1).
    /// Cannot fails, only panic!/OOM on memory exhaustion.
    pub fn push_front(&mut self, v: T) {
        self.head = Some(Box::new(Node(v, self.head.take())));
        self.len += 1;
    }

    /// Pop a element from the front of the list in O(1).
    /// Returns None if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            let Node(v, tail_link) = *{node};
            self.head = tail_link;
            self.len -= 1;
            v
        })
    }

    /// Push an element at the end of the list in O(n).
    /// Cannot fails, only panic!/OOM on memory exhaustion.
    pub fn push_back(&mut self, v: T) {
        *self.last_link() = Some(Box::new(Node(v, None)));
        self.len += 1;
    }

    /// Pop an element from the end of the list in O(n).
    /// Returns None if the list is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        let last_node = {
            if let Some(penultimate_link) = self.penultimate_link() {
                penultimate_link.take()
            } else {
                return None;
            }
        };
        last_node.map(|last_node| {
            let Node(value, _) = *last_node;
            self.len -= 1;
            value
        })
    }
   
    /// Clear the list in O(n).
    pub fn clear(&mut self) {
        while let Some(node) = self.head.take() {
            let Node(_, next_link) = *node;
            self.head = next_link;
        }
    }
}

#[test]
fn basics() {
    let mut l = List::new();
    assert_eq!(l.len(), 0);
    assert!(l.is_empty());
    l.push_back(10);
    assert_eq!(l.len(), 1);
    l.push_back(15);
    l.push_back(20);
    assert_eq!(l.len(), 3);
    assert_eq!(l.pop_back(), Some(20));
    assert_eq!(l.len(), 2);
    assert_eq!(l.pop_front(), Some(10));
    assert_eq!(l.len(), 1);
    l.push_front(5);
    assert_eq!(l.len(), 2);

    assert_eq!(*l.front().unwrap(), 5);
    assert_eq!(*l.back().unwrap(), 15);

    *l.front_mut().unwrap() = 50;
    *l.back_mut().unwrap() = 150;

    assert_eq!(l.pop_back(), Some(150));
    assert_eq!(l.len(), 1);
    assert_eq!(l.pop_front(), Some(50));
    assert_eq!(l.len(), 0);
}

#[cfg(test)]
mod benchs {

extern crate test;

use ::{List, Node};
use self::test::{Bencher, black_box};

static BIGLIST_SIZE: usize = 1024*1024;

fn make_biglist() -> List<i64> {
    let mut l = List::new();
    for i in (0..BIGLIST_SIZE).rev() {
        l.push_front(i as i64);
    }
    return l;
}

#[test]
fn biglist() {
    let l = make_biglist();
    assert_eq!(l.len(), BIGLIST_SIZE);
}

#[test]
fn one_penultimate_safe() {
    let mut l = make_biglist();
    let link = l.penultimate_link().unwrap();
    let node = link.as_ref().unwrap();
    let Node(ref value, _) = **node;
    assert_eq!(*value, BIGLIST_SIZE as i64 -1);
}

#[test]
fn one_penultimate_with_unsafe() {
    let mut l = make_biglist();
    let link = l.penultimate_link_with_unsafe().unwrap();
    let node = link.as_ref().unwrap();
    let Node(ref value, _) = **node;
    assert_eq!(*value, BIGLIST_SIZE as i64 -1);
}


#[bench]
fn penultimate_safe(b: &mut Bencher) {
    let mut l = make_biglist();
    b.iter(|| {
        black_box(l.penultimate_link());
    });
}

#[bench]
fn penultimate_with_unsafe(b: &mut Bencher) {
    let mut l = make_biglist();
    b.iter(|| {
        black_box(l.penultimate_link_with_unsafe());
    });
}

}

