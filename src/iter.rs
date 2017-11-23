use {Link, List};

/// Read-only iterator over a list.
// Can't use derive(Clone) here because it will require an extra Clone bound for
// T which we don't need.
// #[derive(Clone)]
pub struct ListIter<'a, T: 'a> {
    next_link: &'a Link<T>,
    len: usize,
}

impl<'a, T> Clone for ListIter<'a, T> {
    fn clone(&self) -> ListIter<'a, T> {
        ListIter {
            next_link: self.next_link,
            len: self.len,
        }
    }
}

impl<T> List<T> {
    /// Returns an iterator over the list yielding read-only references.
    pub fn iter(&self) -> ListIter<T> {
        ListIter {
            next_link: &self.head,
            len: self.len,
        }
    }
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref node) = *self.next_link {
            self.next_link = &node.next;
            self.len -= 1;
            Some(&node.value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len;
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for ListIter<'a, T> {}

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

#[test]
fn ref_iter_clone() {
    struct NotClonable(usize);
    let mut l = List::new();
    for i in 1..10 {
        l.push_back(NotClonable(i));
    }
    assert_eq!(l.len(), 9);
    let i = l.iter();
    let i2 = i.clone();

    for j in &mut [i, i2] {
        let mut acc = 0;
        for v in j {
            acc += v.0;
        }
        assert_eq!(acc, 45);
        assert_eq!(l.len(), 9);
    }
}
