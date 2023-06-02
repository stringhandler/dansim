use std::cmp::Ordering;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Shard(pub u32);

#[derive(Clone, Debug)]
pub struct Transaction {
    pub id: u32,
    pub shards: Vec<Shard>,
    pub effective_fee: u32,
}

impl PartialEq<Self> for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.effective_fee == other.effective_fee
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.effective_fee.partial_cmp(&other.effective_fee)
    }
}

impl Eq for Transaction {}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.effective_fee.cmp(&other.effective_fee)
    }
}
