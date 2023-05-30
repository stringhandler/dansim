#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Shard(pub u32);


#[derive(Clone, Debug)]
pub struct Transaction {
    pub shards: Vec<Shard>
}
