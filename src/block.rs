use crate::qc::Qc;
use crate::transaction::{Shard, Transaction};
use itertools::Itertools;
use std::sync::Arc;

#[derive(Debug)]
pub struct Block {
    pub id: u32,
    pub parent_id: u32,
    pub shard: Shard,
    pub justify: Arc<Qc>,
    pub height: u32,
    pub proposed_by: u32,
    pub prepare_txs: Vec<Arc<Transaction>>,
    pub precommit_txs: Vec<Arc<Transaction>>,
    pub commit_txs: Vec<Arc<Transaction>>,
}

impl Block {
    pub fn new(
        id: u32,
        parent_id: u32,
        shard: Shard,
        justify: Arc<Qc>,
        height: u32,
        proposed_by: u32,
        prepare_txs: Vec<Arc<Transaction>>,
        precommit_txs: Vec<Arc<Transaction>>,
        commit_txs: Vec<Arc<Transaction>>,
    ) -> Self {
        Self {
            id,
            parent_id,
            shard,
            justify,
            height,
            proposed_by,
            prepare_txs,
            precommit_txs,
            commit_txs,
        }
    }

    pub fn genesis() -> Self {
        Self {
            id: 0,
            parent_id: 0,
            shard: Shard(0),
            justify: Arc::new(Qc::genesis()),
            height: 0,
            proposed_by: 0,
            prepare_txs: vec![],
            precommit_txs: vec![],
            commit_txs: vec![],
        }
    }

    pub fn involved_shards(&self) -> Vec<Shard> {
        let mut res = vec![];
        for tx in self.prepare_txs.iter() {
            for shard in tx.shards.iter() {
                if !res.contains(shard) {
                    res.push(*shard);
                }
            }
        }
        for tx in self.precommit_txs.iter() {
            for shard in tx.shards.iter() {
                if !res.contains(shard) {
                    res.push(*shard);
                }
            }
        }

        res.iter().unique().cloned().collect()
    }
}
