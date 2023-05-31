use std::sync::Arc;
use crate::block::Block;

pub struct Qc {
    pub block_id: u32
}

impl Qc {
    pub fn genesis() -> Self {
        Self {
            block_id: 0
        }
    }
}
