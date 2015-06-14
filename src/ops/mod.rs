use std::{ptr};
use ::{List, Link};

mod core;
mod access;
mod extra;
mod stdtraits;

/// cna you see see?
impl<T> List<T> {
    fn last_link(&mut self) -> &mut Link<T> {
        let mut head_link = &mut self.head;
        loop {
            match *{head_link} {
                Some(ref mut node) => {
                    head_link = &mut node.next;
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
        let mut prev_link = ptr::null_mut();
        let mut head_link: *mut _ = &mut self.head;
        unsafe {
            while let Some(ref mut node) = *head_link {
                prev_link = head_link;
                head_link = &mut node.next;
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
            let found_last_node = {
                if let Some(ref next_node) = node.next {
                    next_node.next.is_none()
                } else {
                    false
                }
            };
            if found_last_node {
                return Some(&mut node.next);
            } else {
                head_link = &mut node.next;
            }
        }
        None
    }
}

#[cfg(test)]
mod benchs {

extern crate test;

use ::{List};
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
    assert_eq!(node.value, BIGLIST_SIZE as i64 -1);
}

#[test]
fn one_penultimate_with_unsafe() {
    let mut l = make_biglist();
    let link = l.penultimate_link_with_unsafe().unwrap();
    let node = link.as_ref().unwrap();
    assert_eq!(node.value, BIGLIST_SIZE as i64 -1);
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
