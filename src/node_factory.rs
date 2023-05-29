use crate::node_id::NodeId;

pub struct NodeFactory {
    next_id: usize
}

impl NodeFactory {
    fn new() -> Self {
        Self {
            next_id: 0
        }
    }

    fn next(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }
}
