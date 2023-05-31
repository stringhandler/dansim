use crate::qc::Qc;
use crate::transaction::Transaction;
use std::sync::Arc;

#[derive(Debug)]
pub struct Block {
    pub id: u32,
    pub parent_id: u32,
    pub justify: Arc<Qc>,
    pub height: u32,
    pub proposed_by: u32,
}

impl Block {
    pub fn new(id: u32, parent_id: u32, justify: Arc<Qc>, height: u32, proposed_by: u32) -> Self {
        Self {
            id,
            parent_id,
            justify,
            height,
            proposed_by,
        }
    }

    pub fn genesis() -> Self {
        Self {
            id: 0,
            parent_id: 0,
            justify: Arc::new(Qc::genesis()),
            height: 0,
            proposed_by: 0,
        }
    }
}
