use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Shard(pub u32);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Transaction {
    pub id: u32,
    pub shards: Vec<Shard>,
    pub effective_fee: u32,
}

#[derive(Debug)]
pub struct SortableByFeeTransaction {
    pub tx: Arc<Transaction>,
    pub fee: u32,
}

impl PartialEq<Self> for SortableByFeeTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.fee == other.fee && self.tx.id == other.tx.id
    }
}

impl PartialOrd for SortableByFeeTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.fee.partial_cmp(&other.fee).and_then(|o| {
            if o == Ordering::Equal {
                self.tx.id.partial_cmp(&other.tx.id)
            } else {
                Some(o)
            }
        })
    }
}

impl Eq for SortableByFeeTransaction {}

impl Ord for SortableByFeeTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fee.cmp(&other.fee).then(self.tx.id.cmp(&other.tx.id))
    }
}
