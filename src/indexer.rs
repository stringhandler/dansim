use crate::node_id::NodeId;

pub struct Indexer {
    id: NodeId
}

impl Indexer {
    fn new(id: NodeId) -> Self {
        Self {
            id
        }
    }
}
