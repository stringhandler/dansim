use crate::transaction::Shard;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CommitteeManager {
    inner: Arc<RwLock<CommitteeManagerInner>>,
}

impl CommitteeManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CommitteeManagerInner {
                committees: Default::default(),
            })),
        }
    }

    pub async fn add_validator(&self, shard: Shard, id: u32) {
        let mut inner = self.inner.write().await;
        let committee = inner.committees.entry(shard).or_insert_with(|| Vec::new());
        committee.push(id);
        committee.sort();
    }

    pub async fn get_committee(&self, shard: Shard) -> Vec<u32> {
        let inner = self.inner.read().await;
        inner.get_committee(shard)
    }

    pub async fn next_leader(&self, shard: Shard, current_leader: u32) -> u32 {
        let committee = self.get_committee(shard).await;
        // special case for genesis block
        if current_leader == 0 {
            return committee[0];
        }
        let index = committee.iter().position(|x| *x == current_leader).unwrap();
        let next_index = (index + 1) % committee.len();
        committee[next_index]
    }
}

#[derive(Debug)]
struct CommitteeManagerInner {
    committees: HashMap<Shard, Vec<u32>>,
}

impl CommitteeManagerInner {
    pub fn get_committee(&self, shard: Shard) -> Vec<u32> {
        self.committees.get(&shard).unwrap().clone()
    }
}
