use crate::transaction::{Shard, Transaction};

pub struct TransactionGenerator {
    static_transactions: Vec<Transaction>,
    current_index: usize
}

impl TransactionGenerator {
    pub fn new() -> Self {
        Self {
            static_transactions : vec![
                Transaction {
                    shards: vec![Shard(0), Shard(1), Shard(2)]
                },
                Transaction {
                    shards: vec![Shard(0),Shard(1)]
                }
            ],
            current_index : 0
        }
    }

    pub fn next(&mut self) -> Option<Transaction> {
        if self.current_index >= self.static_transactions.len() {
            return None;
        }
        let transaction = self.static_transactions[self.current_index].clone();
        self.current_index = self.current_index + 1;
        Some(transaction)
    }
}
