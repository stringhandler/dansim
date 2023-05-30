pub struct Block {
    pub id: u32,
    pub parent_id: u32,
}


impl Block {
    pub fn new(id: u32, parent_id: u32) -> Self {
        Self {
            id,
            parent_id
        }
    }

    pub fn genesis() -> Self {
        Self {
            id: 0,
            parent_id: 0
        }
    }
}
