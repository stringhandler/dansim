pub struct MessageIdFactory {
    next_id: usize
}

impl MessageIdFactory {
    pub fn new() -> Self {
        Self {
            next_id: 0
        }
    }

    pub fn next(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
