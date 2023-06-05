pub struct BlockFactory {
    next_id: u32
}

impl BlockFactory {
    pub fn new() -> Self {
        Self {
            next_id: 0
        }
    }

    pub fn next(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
