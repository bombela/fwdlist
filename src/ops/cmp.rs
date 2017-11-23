use std::cmp::Ordering;
use std::cmp::Ordering::*;
use List;

impl<A: PartialEq> PartialEq for List<A> {
    fn eq(&self, other: &List<A>) -> bool {
        self.len() == other.len() && {
            for (a, b) in self.iter().zip(other.iter()) {
                if a != b {
                    return false;
                }
            }
            true
        }
    }
}

impl<A: Eq> Eq for List<A> {}

impl<A: PartialOrd> PartialOrd for List<A> {
    fn partial_cmp(&self, other: &List<A>) -> Option<Ordering> {
        let (mut a, mut b) = (self.iter(), other.iter());
        loop {
            match (a.next(), b.next()) {
                (None, None) => return Some(Equal),
                (None, _) => return Some(Less),
                (_, None) => return Some(Greater),
                (Some(x), Some(y)) => match x.partial_cmp(&y) {
                    Some(Equal) => (),
                    non_eq => return non_eq,
                },
            }
        }
    }
}

impl<A: Ord> Ord for List<A> {
    fn cmp(&self, other: &List<A>) -> Ordering {
        let (mut a, mut b) = (self.iter(), other.iter());
        loop {
            match (a.next(), b.next()) {
                (None, None) => return Equal,
                (None, _) => return Less,
                (_, None) => return Greater,
                (Some(x), Some(y)) => match x.cmp(&y) {
                    Equal => (),
                    non_eq => return non_eq,
                },
            }
        }
    }
}
