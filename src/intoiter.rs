use ::{List};

/// Iterator consuming a list.
#[derive(Clone)]
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

impl<T> ExactSizeIterator for ListIntoIter<T> {}

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
