use ::{List, Node};

/// Some accessors to front/back elements.
impl<T> List<T> {
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
