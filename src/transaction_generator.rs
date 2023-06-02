use crate::id_provider::IdProvider;
use crate::transaction::{Shard, Transaction};

pub struct TransactionGenerator {
    id_provider: IdProvider,
    static_transactions: Vec<Transaction>,
    current_index: usize,
}

impl TransactionGenerator {
    pub fn new(id_provider: IdProvider) -> Self {
        Self {
            static_transactions: vec![
                Transaction {
                    id: id_provider.next(),
                    shards: vec![Shard(0), Shard(1)],
                    effective_fee: 1,
                },
                Transaction {
                    id: id_provider.next(),
                    shards: vec![Shard(0)],
                    effective_fee: 2,
                },
            ],
            id_provider,
            current_index: 0,
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
