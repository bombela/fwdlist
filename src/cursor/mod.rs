use std::mem;
use ::{List,Link,Node};

pub struct Cursor<'a, T: 'a> {
    next_link: &'a mut Link<T>,
    list_len: &'a mut usize,
    position: usize,
}

impl<T> List<T> {
    pub fn cursor(&mut self) -> Cursor<T> {
        Cursor{
            position: 0,
            list_len: &mut self.len,
            next_link: &mut self.head,
        }
    }
}

impl<'a, T> Cursor<'a, T> {
    pub fn value(&self) -> Option<&T> {
        self.next_link.as_ref().map(|node| {
            &node.value
        })
    }

    pub fn value_mut(&mut self) -> Option<&mut T> {
        self.next_link.as_mut().map(|node| {
            &mut node.value
        })
    }

    pub fn next(&mut self) -> bool {
        let next_link: *mut _ = self.next_link;
        unsafe {
            if let Some(ref mut node) = *next_link {
                self.next_link = &mut node.next;
                self.position += 1;
            }
        }
        self.next_link.is_some()
    }

    pub fn len(&self) -> usize {
        if cfg!(test) {
            self.list_len.checked_sub(self.position).expect("len underflow")
        } else {
            *self.list_len - self.position
        }
    }

    pub fn position(&self) -> usize { self.position }

    pub fn checkpoint(&mut self) -> Cursor<T> {
        Cursor{
            position: self.position,
            list_len: self.list_len,
            next_link: self.next_link,
        }
    }

    // O(min(nth, self.len))
    pub fn nth(&mut self, nth: usize) -> usize {
        // TODO rewrite without self.next.
        let mut nthped = 0;
        while nthped != nth && self.value().is_some() {
            self.next();
            nthped += 1;
        }
        nthped
    }

    // O(self.len - 1)
    pub fn last(&mut self) -> usize {
        match self.len().checked_sub(1) {
            Some(nth) => self.nth(nth),
            None => 0,
        }
    }

    // O(self.len)
    pub fn end(&mut self) -> usize {
        let nth = self.len();
        self.nth(nth)
    }

    // O(1)
    pub fn insert(&mut self, v: T) -> &mut T {
        let mut new_node = Node::new_boxed(v, self.next_link.take());
        let value: *mut _ = &mut new_node.value;
        let next: *mut _ = &mut new_node.next;
        *self.next_link = Some(new_node);
        *self.list_len += 1;
        self.position += 1;
        unsafe {
            self.next_link = &mut *next;
            &mut *value
        }
    }

    // O(1)
    pub fn remove(&mut self) -> Option<T> {
        self.next_link.take().map(|mut node| {
            *self.next_link = node.next.take();
            *self.list_len -= 1;
            node.value
        })
    }

    // O(1)
    pub fn truncate(&mut self) -> List<T> {
        let tail_link = self.next_link.take();
        let tail_len = self.len();
        *self.list_len -= tail_len;
        List {
            len: tail_len,
            head: tail_link,
        }
    }

    fn assign_tail(&mut self, tail: &mut List<T>) {
        if cfg!(test) {
            assert!(self.next_link.is_none());
        }
        *self.next_link = tail.head.take();
        *self.list_len += mem::replace(&mut tail.len, 0);
    }

    // O(other.len()) if self.len > 0
    // O(1) if self.len == 0
    pub fn splice(&mut self, other: &mut List<T>) {
        let tail = self.truncate();
        self.assign_tail(other);
        self.end();
        self.assign_tail(&mut {tail});
    }

    // O(min(at, self.len))
    pub fn split(&mut self, after: usize) -> List<T> {
        let mut c = self.checkpoint();
        c.nth(after);
        c.truncate()
    }

    // O(min(count, self.len))
    pub fn remove_n(&mut self, count: usize) -> List<T> {
        let tail = self.split(count);
        let removed = self.truncate();
        self.assign_tail(&mut {tail});
        removed
    }
}

pub struct CursorIntoIter<'a, T: 'a> {
    cursor: Cursor<'a, T>,
    first: bool,
}


impl<'a, T> IntoIterator for Cursor<'a, T> {
    type Item = &'a mut Cursor<'a, T>;
    type IntoIter = CursorIntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CursorIntoIter{
            cursor: self,
            first: true,
        }
    }
}

impl<'a, T> Iterator for CursorIntoIter<'a, T> {
    type Item = &'a mut Cursor<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first && !self.cursor.next() {
            None
        } else {
            self.first = false;
            unsafe {
                Some(mem::transmute(&mut self.cursor))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.cursor.len();
        (len, Some(len))
    }
}

pub struct CursorIterMut<'c, 'l: 'c, T: 'l> {
    cursor: &'c mut Cursor<'l, T>,
    first: bool,
}

impl<'c, 'l, T> Cursor<'l, T> {

    fn iter_mut(&'c mut self) -> CursorIterMut<'c, 'l, T> {
        CursorIterMut{
            cursor: self,
            first: true,
        }
    }
}

impl<'c, 'l, T> IntoIterator for &'c mut Cursor<'l, T> {
    type Item = &'c mut Cursor<'l, T>;
    type IntoIter = CursorIterMut<'c, 'l, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'c, 'l, T> Iterator for CursorIterMut<'c, 'l, T> {
    type Item = &'c mut Cursor<'l, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first && !self.cursor.next() {
            None
        } else {
            self.first = false;
            unsafe {
                let cursor: *mut _ = self.cursor;
                Some(&mut *cursor)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.cursor.len();
        (len, Some(len))
    }
}

#[test]
fn minimal() {
    let mut l = ::list(0..10);
    let mut i = 0;
    for c in l.cursor() {
        assert_eq!(c.value(), Some(&i));
        i += 1;
    }
    assert_eq!(i, 10);
}

#[test]
fn next() {
    let mut l = ::list(0..10);
    let mut i = 0;
    for mut c in l.cursor() {
        assert_eq!(c.len(), 10-i);
        assert_eq!(c.value(), Some(&i));
        assert_eq!(c.value_mut(), Some(&mut i));
        assert_eq!(c.next(), true);
        i += 1;
        assert_eq!(c.len(), 10-i);
        assert_eq!(c.value(), Some(&i));
        assert_eq!(c.value_mut(), Some(&mut i));
        i += 1;
    }
    assert_eq!(i, 10);
}

#[test]
fn checkpoint() {
    let mut l = ::list(0..10);
    let mut i = 0;
    for mut c in l.cursor() {
        assert_eq!(c.value(), Some(&i));
        {
            let c2 = c.checkpoint();
            let mut j = i;
            for c2 in c2 {
                assert_eq!(c2.value(), Some(&j));
                j += 1;
            }
            assert_eq!(j, 10);
        }
        i += 1;
    }
    assert_eq!(i, 10);
}

#[test]
fn nth() {
    let mut l = ::list(0..10);
    let mut c = l.cursor();

    assert_eq!(c.nth(3), 3);
    assert_eq!(c.value(), Some(&3));

    assert_eq!(c.nth(0), 0);
    assert_eq!(c.value(), Some(&3));

    assert_eq!(c.nth(6), 6);
    assert_eq!(c.value(), Some(&9));

    assert_eq!(c.nth(3), 1);
    assert_eq!(c.value(), None);
}

#[test]
fn last() {
    let mut l = ::list(0..8);
    let mut c = l.cursor();

    assert_eq!(c.last(), 7);
    assert_eq!(c.value(), Some(&7));
}

#[test]
fn end() {
    let mut l = ::list(0..6);
    let mut c = l.cursor();

    assert_eq!(c.end(), 6);
    assert_eq!(c.value(), None);
}

#[test]
fn insert() {
    let mut l = ::list(0..4);
    {
        let mut c = l.cursor();

        assert_eq!(c.len(), 4);
        assert_eq!(c.position(), 0);
        assert_eq!(c.value(), Some(&0));

        c.insert(42);

        assert_eq!(c.len(), 4);
        assert_eq!(c.position(), 1);
        assert_eq!(c.value(), Some(&0));
    }
    assert_eq!(l.len(), 5);
    assert_eq!(l, ::list([42, 0, 1, 2, 3].iter().cloned()));
    {
        let mut c = l.cursor();
        assert_eq!(c.len(), 5);
        assert_eq!(c.position(), 0);
        assert_eq!(c.value(), Some(&42));

        assert_eq!(c.next(), true);

        assert_eq!(c.len(), 4);
        assert_eq!(c.position(), 1);
        assert_eq!(c.value(), Some(&0));

        c.insert(43);

        assert_eq!(c.len(), 4);
        assert_eq!(c.position(), 2);
        assert_eq!(c.value(), Some(&0));
    }
    assert_eq!(l.len(), 6);
    assert_eq!(l, ::list([42, 43, 0, 1, 2, 3].iter().cloned()));
    {
        let mut c = l.cursor();
        assert_eq!(c.len(), 6);
        assert_eq!(c.position(), 0);
        assert_eq!(c.value(), Some(&42));

        for _ in 0..5 {
            assert_eq!(c.next(), true);
        }

        assert_eq!(c.len(), 1);
        assert_eq!(c.position(), 5);
        assert_eq!(c.value(), Some(&3));

        c.insert(44);

        assert_eq!(c.len(), 1);
        assert_eq!(c.position(), 6);
        assert_eq!(c.value(), Some(&3));
    }
    assert_eq!(l.len(), 7);
    assert_eq!(l, ::list([42, 43, 0, 1, 2, 44, 3].iter().cloned()));
    {
        let mut c = l.cursor();
        assert_eq!(c.len(), 7);
        assert_eq!(c.position(), 0);
        assert_eq!(c.value(), Some(&42));

        for _ in 0..6 {
            assert_eq!(c.next(), true);
        }
        assert_eq!(c.next(), false);

        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 7);
        assert_eq!(c.value(), None);

        assert_eq!(c.next(), false);

        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 7);
        assert_eq!(c.value(), None);

        c.insert(45);

        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 8);
        assert_eq!(c.value(), None);

        c.insert(46);

        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 9);
        assert_eq!(c.value(), None);
    }
    assert_eq!(l.len(), 9);
    assert_eq!(l, ::list([42, 43, 0, 1, 2, 44, 3, 45, 46].iter().cloned()));
}

#[test]
fn append() {
    let mut l = List::new();
    {
        let mut c = l.cursor();
        for i in 0..15 {
            c.insert(i);
        }
        assert_eq!(c.len(), 0);
    }
    assert_eq!(l.len(), 15);
    assert_eq!(l, ::list(0..15));
}

#[test]
fn remove() {
    let mut l = ::list(0..10);
    {
        let mut c = l.cursor();
        assert_eq!(c.len(), 10);
        assert_eq!(c.position(), 0);
        assert_eq!(c.remove(), Some(0));
        assert_eq!(c.len(), 9);
        assert_eq!(c.position(), 0);
        let mut i = 1;
        let mut pos = 0;
        for c in &mut c {
            assert_eq!(c.len(), 10-i);
            assert_eq!(c.position(), pos);
            assert_eq!(c.value(), Some(&i));
            if i == 5 {
                assert_eq!(c.remove(), Some(i));
                i += 1;
                pos += 0;
            }
            i += 1;
            pos += 1;
        }
        assert_eq!(c.len(), 0);
    }
    assert_eq!(l.len(), 8);
}


#[test]
fn truncate() {
    let mut a = ::list(0..20);
    let mut b;
    {
        let mut c = a.cursor();
        assert_eq!(c.nth(10), 10);
        assert_eq!(c.len(), 10);
        b = c.truncate();
        assert_eq!(c.len(), 0);
    }
    assert_eq!(a, ::list(0..10));
    assert_eq!(b, ::list(10..20));
    let mut c;
    {
        let mut bc = b.cursor();
        assert_eq!(bc.nth(9), 9);
        assert_eq!(bc.len(), 1);
        c = bc.truncate();
        assert_eq!(bc.len(), 0);
    }
    assert_eq!(b, ::list(10..19));
    assert_eq!(c, ::list(19..20));
    let d;
    {
        let mut cc = c.cursor();
        assert_eq!(cc.nth(1), 1);
        assert_eq!(cc.len(), 0);
        d = cc.truncate();
        assert_eq!(cc.len(), 0);
    }
    assert_eq!(c, ::list(19..20));
    assert_eq!(d, ::list(0..0));
}


#[test]
fn splice() {
    let mut a = ::list(0..5);
    let mut b = ::list(30..35);
    {
        let mut c = a.cursor();
        c.nth(3);
        assert_eq!(c.len(), 2);
        assert_eq!(c.position(), 3);
        c.splice(&mut b);
        assert_eq!(c.len(), 2);
        assert_eq!(c.position(), 3+5);
    }
    assert_eq!(a.len(), 10);
    assert_eq!(a, ::list(
            [ 0, 1, 2, 30, 31, 32, 33, 34, 3, 4].iter().cloned()));
    assert_eq!(b, ::list(0..0));
    {
        let mut c = b.cursor();
        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 0);
        c.splice(&mut ::list(10..15));
        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 5);
    }
    assert_eq!(b.len(), 5);
    assert_eq!(b, ::list(10..15));
    {
        let mut c = b.cursor();
        assert_eq!(c.len(), 5);
        assert_eq!(c.position(), 0);
        c.splice(&mut ::list(5..10));
        assert_eq!(c.len(), 5);
        assert_eq!(c.position(), 5);
    }
    assert_eq!(b.len(), 10);
    assert_eq!(b, ::list(5..15));
    {
        let mut c = b.cursor();
        assert_eq!(c.len(), 10);
        assert_eq!(c.position(), 0);
        c.end();
        c.splice(&mut ::list(15..20));
        assert_eq!(c.len(), 0);
        assert_eq!(c.position(), 15);
    }
    assert_eq!(b.len(), 15);
    assert_eq!(b, ::list(5..20));
}

#[test]
fn split() {
    let mut a = ::list(0..20);
    let b;
    {
        let mut c = a.cursor();
        assert_eq!(c.len(), 20);
        assert_eq!(c.position(), 0);
        b = c.split(10);
        assert_eq!(c.len(), 10);
        assert_eq!(c.position(), 0);
    }
    assert_eq!(a.len(), 10);
    assert_eq!(a, ::list( 0..10));

    assert_eq!(b.len(), 10);
    assert_eq!(b, ::list(10..20));
}


#[test]
fn remove_n() {
    let mut l = ::list(0..10);
    {
        let mut c = l.cursor();
        assert_eq!(c.position(), 0);
        assert_eq!(c.len(), 10);
        c.nth(3);
        assert_eq!(c.position(), 3);
        assert_eq!(c.len(), 7);
        assert_eq!(c.remove_n(4), ::list(3..7));
        assert_eq!(c.position(), 3);
        assert_eq!(c.len(), 3);
    }
    assert_eq!(l, ::list([0, 1, 2, 7, 8, 9].iter().cloned()));
}


//#[test]
//fn merge_sort() {
    //let mut l = ::list((0..10).rev());

    //let mut k = 1;
    //loop {
        //println!("-**- {:?} ({:?})", l, k);
        //let mut tail = l;
        //let mut merge_counts = 0;
        //{
            //l = List::new();
            ////let mut cl = l.cursor();

            //let mut a;
            //let mut b;
            //while !tail.is_empty() {
                //a = tail;
                //b = a.split_off(k);
                //tail = b.split_off(k);

                //let mut ca = a.cursor();
                //let mut cb = b.cursor();

                //loop {
                    //let a_smaller;
                    //if let (Some(va), Some(vb)) = (ca.value(), cb.value()) {
                        //println!("{:?} - {:?}", va, vb);
                        //a_smaller = va <= vb;
                    //} else {
                        //println!("--");
                        //break;
                    //}
                    //if a_smaller {
                        ////cl.insert_all(ca.remove_n(1));
                        //l.append(&mut ca.remove_n(1));
                        //ca.next();
                    //} else {
                        //l.append(&mut cb.remove_n(1));
                        //cb.next();
                    //}
                //}
                ////cl.insert_all(ca.truncate());
                ////cl.insert_all(cb.truncate());
                //l.append(&mut ca.truncate());
                //l.append(&mut cb.truncate());

                //merge_counts += 1;
                //println!("----- {:?}", merge_counts);
            //}

        //}
        //println!("-+++++++++- {:?} {:?} {:?}", merge_counts, l, l.len());
        //if merge_counts <= 1 {
            //break;
        //}

        //k *= 2;
    //}

    //println!("---> {:?}", l);
    //for (i, v) in l.iter().enumerate() {
        //assert_eq!(i, *v);
    //}
//}
