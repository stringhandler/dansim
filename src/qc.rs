use crate::block::Block;
use std::sync::Arc;

#[derive(Debug)]
pub struct Qc {
    pub id: u32,
    pub block_id: u32,
    pub votes: Vec<u32>,
    pub block_height: u32,
}

impl Qc {
    pub fn new(id: u32, block_id: u32, block_height: u32, votes: Vec<u32>) -> Self {
        Self {
            id,
            block_id,
            block_height,
            votes,
        }
    }
    pub fn genesis() -> Self {
        Self {
            id: 0,
            block_id: 0,
            block_height: 0,
            votes: vec![],
        }
    }
}
