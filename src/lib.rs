//! A simple forward linked list.
//!
//! It's a linked list. Its not cache friendly, its relatively slow when you think about it, but it
//! allows for O(1) insertion... after the current iterator location, maybe you care about that.
//!
//! # Avoiding unsafe
//! The goal here is to play with Rust and see how much unsafe is needed. It turns out that you can
//! implement everything but the mutable iterator without using unsafe.
//!
//! The mutable iterator needs unsafe only because it returns a mutable reference with a different
//! lifetime than the mutable reference on the iterator itself. The compiler cannot infer that
//! auto-magically and needs a bit of our help.
//!
//! # penultimate_link() performances
//!
//! Sometimes the code feels a more convoluted than necessary to please the borrow checker.  Some
//! unsafe code would make the code not only easier to read, but also *we naively believe*, more
//! efficient for the machine.
//!
//! The best example here is `penultimate_link()`, which returns a mutable reference to the
//! last but one link of the list.
//!
//! To illustrate what this function returns, let's assume the following list:
//!
//! ```text
//! head_link -> node1.next -> node2.next -> node3.next -> nil
//! ```
//!
//! In this case, `penultimate_link()` will return a mutable reference to `node2.next`. It is then
//! trivial to implement `pop_back()` with a simple `Option.take()`.
//!
//! See `penultimate_link()` and `penultimate_link_with_unsafe()` implementations further below.
//!
//! ## Assembly output
//!
//! Take a look at the assembly outputs (cargo build --release) below:
//!
//! * `penultimate_link()`:
//!
//! ```gas
//! 0000000000016200 <::only_safe::>:
//!    16200:   48 8b 4f 08             mov    0x8(%rdi),%rcx
//!    16204:   31 c0                   xor    %eax,%eax
//!    16206:   48 85 c9                test   %rcx,%rcx
//!    16209:   74 1f                   je     1622a <::only_safe::+0x2a>
//!    1620b:   31 c0                   xor    %eax,%eax
//!    1620d:   0f 1f 00                nopl   (%rax)
//!    16210:   48 89 ca                mov    %rcx,%rdx
//!    16213:   48 8b 4a 08             mov    0x8(%rdx),%rcx
//!    16217:   48 85 c9                test   %rcx,%rcx
//!    1621a:   74 0e                   je     1622a <::only_safe::+0x2a>
//!    1621c:   48 83 79 08 00          cmpq   $0x0,0x8(%rcx)
//!    16221:   75 ed                   jne    16210 <::only_safe::+0x10>
//!    16223:   48 83 c2 08             add    $0x8,%rdx
//!    16227:   48 89 d0                mov    %rdx,%rax
//!    1622a:   c3                      retq
//! ```
//! * `penultimate_link_with_unsafe()`:
//!
//! ```gas
//! 00000000000168a0 <::with_unsafe::>:
//!    168a0:   31 c0                   xor    %eax,%eax
//!    168a2:   48 83 7f 08 00          cmpq   $0x0,0x8(%rdi)
//!    168a7:   74 18                   je     168c1 <::with_unsafe::+0x21>
//!    168a9:   48 83 c7 08             add    $0x8,%rdi
//!    168ad:   0f 1f 00                nopl   (%rax)
//!    168b0:   48 8b 0f                mov    (%rdi),%rcx
//!    168b3:   48 83 79 08 00          cmpq   $0x0,0x8(%rcx)
//!    168b8:   48 89 f8                mov    %rdi,%rax
//!    168bb:   48 8d 79 08             lea    0x8(%rcx),%rdi
//!    168bf:   75 ef                   jne    168b0 <::with_unsafe::+0x10>
//!    168c1:   c3                      retq
//! ```
//! ## Assembly quick analysis
//!
//! The first thing to note, is how well the original code is translated from high level Option and
//! Box to simple null-able pointers.
//!
//! * `penultimate_link()` is a loop with two conditional branches inside, and it tests twice every
//! nodes of the list (exactly like in the Rust code). One test on every next_link, before testing
//! it again when it become the new link to work on new every new iteration.
//! * `penultimate_with_unsafe()` is a loop with only one condition, but it keeps a “prev_link”
//! pointer handy, again like in the Rust code.
//!
//! Looking at the assembly with my ridiculously weak knowledge of modern CPU architecture, I infer
//! that `penultimate_link()` requires twice the amount of branches predictions and both functions
//! perform two data read per iteration.
//!
//! Considering how modern CPUs seems to pipeline/pre-fetch like crazy, the two branchs predictions
//! should pretty much cost like only one.
//!
//! ## Callgrind/Cachegrind (valgrind) analysis
//!
//! After adding `#[inline(never)]` on both `penultimate_link*` functions, I ran valgrind like so:
//!
//! ```sh
//! $ valgrind --tool=callgrind --dump-instr=yes --trace-jump=yes --cache-sim=yes --branch-sim=yes --collect-atstart=no --toggle-collect=*penultimate_link* target/release/fwdlist... --test one_penultimate
//! ```
//! We basically get the following report:
//!
//! | version   | Ir        | Dr        | D1mr      | DLmr    | Bc        | Bcm |
//! |-----------|-----------|-----------|-----------|---------|-----------|-----|
//! | safe_only | 6,291,459 | 2,097,152 | 1 261,697 | 236,874 | 2,097,151 | 4   |
//! | unsafe    | 5,242,886 | 2,097,154 | 1 261,697 | 238,678 | 1,048,577 | 5   |
//!
//! * **Ir**: instruction read, `penultimate_link()` has more instructions and so more instruction
//! read.
//! * **Dr**: data read. `penultimate_with_unsafe()` performs one more loop iteration, reading
//! **2** more data.
//! * **D1mr**: data read misses on L1 cache. Similar between the two.
//! * **DLmr**: data read misses on Last Level cache. Interestingly, `penultimate_with_unsafe()`
//! has more misses.
//! * **Bc**: Conditional branches. Confirms that `penultimate_link()` has two vs one conditions.
//! * **Bcm**: Conditional branches misses. `penultimate_with_unsafe()` gets one more, maybe the
//! extra iteration?
//!
//! ## Benchmark
//!
//! `penultimate_link()` is faster than `penultimate_with_unsafe()` on real hardware.
//!
//! Benchmarks with List\<i64\> and BIGLIST_SIZE=1Mib (list takes ~16Mib):
//!
//! ```text
//! AMD Phenom(tm) II X4 965 Processor
//! penultimate_safe        ... bench:   3651099 ns/iter (+/- 35924)
//! penultimate_with_unsafe ... bench:   3687377 ns/iter (+/- 33386)
//!
//! Intel(R) Core(TM) i7-2720QM CPU @ 2.20GHz
//! penultimate_safe        ... bench:   2333951 ns/iter (+/- 27634)
//! penultimate_with_unsafe ... bench:   2334611 ns/iter (+/- 43642)
//!
//! Intel(R) Core(TM) i5-3320M CPU @ 2.60GHz
//! penultimate_safe        ... bench:   1675111 ns/iter (+/- 106477)
//! penultimate_with_unsafe ... bench:   2127297 ns/iter (+/- 128966)
//! ```
//!
//! Benchmarks with List\<i64\> and BIGLIST_SIZE=1Gib (list takes ~16Gib):
//!
//! ```text
//! Intel(R) Xeon(R) CPU E5-1650 0 @ 3.20GHz
//! penultimate_safe        ... bench: 2399497518 ns/iter (+/- 357540058)
//! penultimate_with_unsafe ... bench: 2509462341 ns/iter (+/- 377119880)
//! ```
//! ## Performances conclusion
//!
//! Convoluted safe code vs simpler unsafe code doesn't necessary mean that unsafe code is going to
//! be faster. In our specific case `penultimate_with_unsafe()` is indeed slower!
//!
//! This is great because with safe Rust code only, the compiler basically proves for us that there
//! is no possible memory bugs. Any code refactoring cannot possibly introduce memory bugs easier,
//! the compiler wouldn't let it pass.
//!
//!
//! Happy hacking!

#![allow(unused_features)]
#![feature(test)]

/// A simply linked list.
pub struct List<T> {
    len: usize,
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T>(T, Link<T>);

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

    fn last_link(&mut self) -> &mut Link<T> {
        let mut head_link = &mut self.head;
        loop {
            match *{head_link} {
                Some(ref mut node) => {
                    let Node(_, ref mut tail_link) = **node;
                    head_link = tail_link;
                },
                ref mut nil_link @ None => {
                    return nil_link;
                },
            }
        }
    }

    #[allow(dead_code)]
    //#[inline(never)] // <- if testing with callgrind.
    fn penultimate_link_with_unsafe(&mut self) -> Option<&mut Link<T>> {
        let mut prev_link = std::ptr::null_mut();
        let mut head_link: *mut _ = &mut self.head;
        unsafe {
            while let Some(ref mut node) = *head_link {
                let Node(_, ref mut next_link) = **node;
                prev_link = head_link;
                head_link = next_link;
            }
            if !prev_link.is_null() {
                Some(&mut *prev_link)
            } else {
                None
            }
        }
    }

    //#[inline(never)] // <- if testing with callgrind.
    fn penultimate_link(&mut self) -> Option<&mut Link<T>> {
        let mut head_link = &mut self.head;
        while let Some(ref mut node) = *{head_link} {
            let Node(_, ref mut tail_link) = **node;
            let found_last_node = {
                if let Some(ref next_node) = *tail_link {
                    let Node(_, ref next_tail_link) = **next_node;
                    next_tail_link.is_none()
                } else {
                    false
                }
            };
            if found_last_node {
                return Some(tail_link);
            } else {
                head_link = tail_link;
            }
        }
        None
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

    /// Returns a reference to the first element in the list.
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|link| {
            let Node(ref v, _) = **link;
            v
        })
    }

    /// Returns a mutable reference to the first element in the list.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|link| {
            let Node(ref mut v, _) = **link;
            v
        })
    }

    /// Returns a reference to the last element in the list.
    pub fn back(&self) -> Option<&T> {
        let mut head_link = &self.head;
        while let Some(ref node) = *head_link {
            let Node(ref v, ref next_link) = **node;
            if next_link.is_none() {
                return Some(v);
            }
            head_link = next_link;
        }
        None
    }

    /// Returns a mutable reference to the last element in the list.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.penultimate_link().and_then(|link| {
            link.as_mut().map(|node| {
                let Node(ref mut v, _) = **node;
                v
            })
        })
    }

}

/// Drop the list in O(n).
impl<T> Drop for List<T> {
    fn drop(&mut self) { self.clear(); }
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

use super::*;
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
    let super::Node(ref value, _) = **node;
    assert_eq!(*value, BIGLIST_SIZE as i64 -1);
}

#[test]
fn one_penultimate_with_unsafe() {
    let mut l = make_biglist();
    let link = l.penultimate_link_with_unsafe().unwrap();
    let node = link.as_ref().unwrap();
    let super::Node(ref value, _) = **node;
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

/// Iterator consuming a list.
pub struct ListIntoIter<T> {
    list: List<T>
}

impl<T> Iterator for ListIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len();
        (len, Some(len))
    }
}

/// `for v in my_list { v ... }`
impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = ListIntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        ListIntoIter{list: self}
    }
}

#[test]
fn into_iter() {
    let mut l = List::new();
    for i in 1..10 {
        l.push_back(i);
    }
    assert_eq!(l.len(), 9);
    let mut acc = 0;
    for v in l {
        acc += v;
    }
    assert_eq!(acc, 45);
}

/// Read-only iterator over a list.
pub struct ListIter<'a, T: 'a> {
    next_link: &'a Link<T>,
    len: usize,
}

impl<T> List<T> {
    /// Returns an iterator over the list yielding read-only references.
    pub fn iter(&self) -> ListIter<T> {
        ListIter{next_link: &self.head, len: self.len}
    }
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref node) = *self.next_link {
            let Node(ref value, ref next_link) = **node;
            self.next_link = next_link;
            self.len -= 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len;
        (len, Some(len))
    }
}

/// `for v in &my_list { *v ... }`
impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = ListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[test]
fn ref_iter() {
    let mut l = List::new();
    for i in 1..10 {
        l.push_back(i);
    }
    assert_eq!(l.len(), 9);
    let mut acc = 0;
    for v in &l {
        acc += *v;
    }
    assert_eq!(acc, 45);
    assert_eq!(l.len(), 9);
}

/// Mutable iterator over a list. Provides few extra functions to modify the list at the point
/// of iteration.
pub struct ListIterMut<'a, T: 'a> {
    next_link: &'a mut Link<T>,
    len: usize,
    list_len: &'a mut usize,
}

impl<T> List<T> {
    /// Returns an iterator over the list yielding mutable references.
    pub fn iter_mut(&mut self) -> ListIterMut<T> {
        ListIterMut{
            len: self.len,
            list_len: &mut self.len,
            next_link: &mut self.head,
        }
    }
}

impl<'a, T> Iterator for ListIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_link: *mut _ = self.next_link;
        unsafe {
            if let Some(ref mut node) = *next_link {
                let Node(ref mut value, ref mut next_link) = **{node};
                self.next_link = next_link;
                self.len -= 1;
                Some(value)
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len;
        (len, Some(len))
    }
}

/// `for v in &mut my_list { *v = ... }`
impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = ListIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[test]
fn mutref_iter() {
    let mut l = List::new();
    for i in 1..10 {
        l.push_back(i);
    }
    assert_eq!(l.len(), 9);
    let mut acc = 0;
    for v in &mut l {
        acc += *v;
        *v += 1;
    }
    assert_eq!(acc, 45);
    assert_eq!(l.len(), 9);

    let mut acc = 0;
    for v in &mut l {
        acc += *v;
        *v += 1;
    }
    assert_eq!(acc, 54);
    assert_eq!(l.len(), 9);
}

impl<'a, T> ListIterMut<'a, T> {
    /// Returns a reference to the next element, without moving the iterator.
    pub fn peek_next(&self) -> Option<&T> {
        self.next_link.as_ref().map(|node| {
            let Node(ref v, _) = **node;
            v
        })
    }

    /// Returns a mutable reference to the next element, without moving the iterator.
    pub fn peek_next_mut(&mut self) -> Option<&mut T> {
        self.next_link.as_mut().map(|node| {
            let Node(ref mut v, _) = **node;
            v
        })
    }

    /// Insert `v` just after the element most recently returned by `.next()` in O(1).
    ///
    /// The inserted element does not appear in the iteration.
    pub fn insert_next(&mut self, v: T) {
        let mut new_node = Box::new(Node(v, self.next_link.take()));
        let tail_link: *mut _ = {
            let Node(_, ref mut tail_link) = *new_node;
            tail_link
        };
        *self.next_link = Some(new_node);
        unsafe {
            self.next_link = &mut *tail_link;
        }
        *self.list_len += 1;
    }

    /// Remove the element after the one most recently returned by `.next()` in O(1);
    ///
    /// Returns the removed value or None if the iterator is already at the end of the list.
    pub fn remove_next(&mut self) -> Option<T> {
        self.next_link.take().map(|node| {
            let Node(v, tail_link) = *{node};
            *self.next_link = tail_link;
            *self.list_len -= 1;
            self.len -= 1;
            v
        })
    }

    /// Truncate the list right after the element most recently return by `.next()` in O(1).
    ///
    /// * returns a new list owning the rest of the truncated list.
    /// * the iterator is now at the end of the freshly truncated list.
    /// * returns an empty list if the iterator is already exhausted.
    pub fn truncate_next(&mut self) -> List<T> {
        let tail_link = self.next_link.take();
        *self.list_len -= self.len;
        List {
            len: std::mem::replace(&mut self.len, 0),
            head: tail_link,
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
    for &i in &[0,1,2,3,42,4,5,150,7,9] {
        assert_eq!(iter.next(), Some(i));
    }
}

impl<T> Default for List<T> {
    fn default() -> List<T> { List::new() }
}

/// Extend the list from the content of `iter` in O(n).
impl<T> Extend<T> for List<T> {
    fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=T> {
        let mut last_link: *mut _ = self.last_link();
        unsafe {
            for v in iter {
                let mut node = Box::new(Node(v, None));
                let next_link: *mut _ = {
                    let Node(_, ref mut next_link_) = *node;
                    next_link_
                };
                *last_link = Some(node);
                self.len += 1;
                last_link = &mut *next_link;
            }
        }
    }
}

use std::iter::FromIterator;
/// Construct a list from the content of the iterator `iter` in O(n).
impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> List<T> {
        let mut r = List::new();
        r.extend(iter);
        r
    }
}

/// Clone a list in O(n).
///
/// `clone_from()` will reuse as many nodes of `self` as possible to avoid reallocation;
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> List<T> {
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, source: &Self) {
        let mut src_iter = source.iter().cloned();
        let mut dst_iter = self.iter_mut();
        let mut src_exhausted = false;
        for dst_v in &mut dst_iter {
            if let Some(v) = src_iter.next() {
                *dst_v = v;
            } else {
                src_exhausted = true;
                break;
            }
        }
        if src_exhausted {
            dst_iter.truncate_next();
        } else {
            for src_v in src_iter {
                dst_iter.insert_next(src_v);
            }
        }
    }
}

use std::fmt;
impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.iter().fold(f.debug_list(), |mut b, e| {
            b.entry(e);
            b
        }).finish()
    }
}

use std::hash::{Hash, Hasher};
impl<T: Hash> Hash for List<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for v in self {
            v.hash(state);
        }
    }
}

impl<T> List<T> {
    /// Moves all elements from `other` to the end of the list in O(self.len());
    pub fn append(&mut self, other: &mut List<T>) {
        *self.last_link() = other.head.take();
        self.len += std::mem::replace(&mut other.len, 0);
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

impl<T> List<T> {
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
            len: std::mem::replace(&mut self.len, at) - at,
            head: tail_link,
        }
    }
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
