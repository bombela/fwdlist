use {Link, List};

mod extra;

/// Mutable iterator over a list.
pub struct ListIterMut<'a, T: 'a> {
    next_link: &'a mut Link<T>,
    len: usize,
    list_len: &'a mut usize,
}

impl<T> List<T> {
    /// Returns an iterator over the list yielding mutable references.
    pub fn iter_mut(&mut self) -> ListIterMut<T> {
        ListIterMut {
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
                let (value, next) = node.take_mut();
                self.next_link = next;
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

impl<'a, T> ExactSizeIterator for ListIterMut<'a, T> {}

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
