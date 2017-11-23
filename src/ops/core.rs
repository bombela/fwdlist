use {List, Node};

impl<T> List<T> {
    /// A new empty list.
    pub fn new() -> List<T> {
        List { len: 0, head: None }
    }

    /// The size of the list in O(1).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if list is empty in O(1);
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Push a new element at the front of the list in O(1).
    /// Cannot fails, only panic!/OOM on memory exhaustion.
    pub fn push_front(&mut self, v: T) {
        self.head = Some(Node::new_boxed(v, self.head.take()));
        self.len += 1;
    }

    /// Pop a element from the front of the list in O(1).
    /// Returns None if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            let (value, next) = node.take();
            self.head = next;
            self.len -= 1;
            value
        })
    }

    /// Push an element at the end of the list in O(n).
    /// Cannot fails, only panic!/OOM on memory exhaustion.
    pub fn push_back(&mut self, v: T) {
        *self.last_link() = Some(Node::new_boxed(v, None));
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
            self.len -= 1;
            last_node.value
        })
    }

    /// Clear the list in O(n).
    pub fn clear(&mut self) {
        while let Some(node) = self.head.take() {
            self.head = node.next;
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
