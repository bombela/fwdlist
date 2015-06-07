//! A simple forward linked list.
//!
//! It's a linked list. Its not cache friendly, its relatively slow when you
//! think about it, but it allows for O(1) insertion... after the current
//! iterator location, maybe you care about that.
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
//
//! Happy hacking!

#![allow(unused_features)]
#![feature(test)]

pub use intoiter::ListIntoIter;
pub use iter::ListIter;
pub use itermut::ListIterMut;

mod ops;
mod intoiter;
mod iter;
mod itermut;

/// A simply linked list.
pub struct List<T> {
    len: usize,
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T>(T, Link<T>);
