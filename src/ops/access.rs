use ::{List};

/// Some accessors to front/back elements.
impl<T> List<T> {
    /// Returns a reference to the first element in the list.
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.value
        })
    }

    /// Returns a mutable reference to the first element in the list.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.value
        })
    }

    /// Returns a reference to the last element in the list.
    pub fn back(&self) -> Option<&T> {
        let mut head_link = &self.head;
        while let Some(ref node) = *head_link {
            if node.next.is_none() {
                return Some(&node.value);
            }
            head_link = &node.next;
        }
        None
    }

    /// Returns a mutable reference to the last element in the list.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.penultimate_link().and_then(|link| {
            link.as_mut().map(|node| {
                &mut node.value
            })
        })
    }
}
