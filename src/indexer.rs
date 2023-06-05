use crate::node_id::NodeId;

pub struct Indexer {
    pub id: u32
}

impl Indexer {
    pub fn new(id: u32) -> Self {
        Self {
            id
        }
    }
}
