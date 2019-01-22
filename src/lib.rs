//! A simple forward linked list.
//!
//! It's a linked list. Its not cache friendly, its relatively slow when you
//! think about it, but it allows for O(1) insertion... after the current
//! cursor location, maybe you care about that.
//!
//! # Trivial example
//! ```
//! use fwdlist::List;
//!
//! let mut q = List::new();
//! q.push_back(2);
//! q.push_front(1);
//! q.push_back(3);
//!
//! println!("{:?}", q);
//! for v in q {
//!     println!("{:?}", v);
//! }
//! ```
//!
//! # Using cursors
//!
//! A cursor helps walking the list while modifying its structure. Not to be
//! confused with a mutable iterator, which merely provides mutable access to
//! values of the list.
//!
//! For example, using cursors, we can implement a non-recursive merge-sort by
//! sorting and merging the list by pair of 2, 4, 8, etc...
//!
//! ```
//! extern crate fwdlist;
//!
//! use fwdlist::List;
//! use std::fmt::Debug;
//!
//! fn mklist<I: Iterator>(i: I) -> List<I::Item> {
//!     i.collect::<List<_>>()
//! }
//!
//! fn merge<'c, T>(mut a: List<T>, mut b: List<T>)
//!     -> List<T> where T: Ord + Debug {
//!     use std::cmp::Ordering::*;
//!
//!     let mut r = List::new();
//!     {
//!         let mut a = a.cursor();
//!         let mut b = b.cursor();
//!         let mut o = r.cursor();
//!         loop {
//!             let cmpr = {
//!                 if let (Some(a), Some(b)) = (a.value(), b.value()) {
//!                     a.cmp(b)
//!                 } else {
//!                     break
//!                 }
//!             };
//!             if cmpr == Less {
//!                 o.splice(&mut a.remove_n(1));
//!             } else {
//!                 o.splice(&mut b.remove_n(1));
//!             }
//!         }
//!         o.splice(&mut a.truncate());
//!         o.splice(&mut b.truncate());
//!     }
//!     r
//! }
//!
//! fn merge_sort<T>(mut l: List<T>) -> List<T> where T: Ord + Debug {
//!     let mut run_len = 1;
//!     let max_run_len = l.len();
//!     while run_len < max_run_len {
//!         let mut tail = l;
//!         l = List::new();
//!         let mut cl = l.cursor();
//!
//!         while !tail.is_empty() {
//!             let mut a = tail;
//!             let mut b = a.cursor().split(run_len);
//!             tail = b.cursor().split(run_len);
//!             cl.splice(&mut merge(a,b));
//!         }
//!         run_len *= 2;
//!     }
//!     return l;
//! }
//!
//! fn main() {
//!     const LMAX: usize = 30;
//!     let mut l = mklist((LMAX/2..LMAX).rev());
//!     l.extend(mklist(0..LMAX/2));
//!     println!("-> {:?}", l);
//!     let l = merge_sort(l);
//!     println!("-> {:?}", l);
//!     assert_eq!(l, mklist(0..LMAX));
//! }
//! ```
//!
//! Happy hacking!

#![cfg_attr(feature = "bench", feature(test))]

pub use crate::intoiter::ListIntoIter;
pub use crate::iter::ListIter;
pub use crate::itermut::ListIterMut;

mod cursor;
mod intoiter;
mod iter;
mod itermut;
mod ops;

/// A simply linked list.
pub struct List<T> {
    len: usize,
    head: Link<T>,
}

/// A cursor to navigate the list and reshape it.
///
/// Conceptually, a cursor moves between nodes, think the cursor of your text
/// editor.
///
/// ## Cursor by example
///
/// ```
/// use fwdlist::List;
///
/// let mut q: List<_> = (0..5).collect();
/// let mut c = q.cursor();
/// ```
/// So given the list `[0,1,2,3,4]`, the cursor `c`, represented by `|`
/// initially points to the first node:
///
/// ```plain
///  |0 1 2 3 4
/// ```
/// After advancing the cursor once `c.advance()`:
///
/// ```plain
///   0|1 2 3 4
/// ```
/// And after Advancing the cursor 4 times `c.nth(4)`:
///
/// ```plain
///   0 1 2 3 4|
/// ```
///
/// # Modifying the structure of the list
///
/// A cursor let you modify the list after its position (the `tail`). A cursor
/// is really an abstraction of a mutable pointer to the next node of the list.
///
/// With a cursor, you can truncate the list, insert and removes nodes, etc.
///
pub struct Cursor<'a, T> {
    next_link: &'a mut Link<T>,
    list_len: &'a mut usize,
    position: usize,
}

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T> Node<T> {
    fn new_boxed(value: T, next: Link<T>) -> Box<Node<T>> {
        Box::new(Node { value, next })
    }

    fn take(self) -> (T, Link<T>) {
        (self.value, self.next)
    }

    fn take_mut(&mut self) -> (&mut T, &mut Link<T>) {
        (&mut self.value, &mut self.next)
    }
}
