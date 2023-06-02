use crate::cli::Cli;
use crate::id_provider::IdProvider;
use crate::transaction::{Shard, Transaction};
use itertools::Itertools;
use rand::{thread_rng, RngCore};
use std::sync::Arc;

pub struct TransactionGenerator {
    id_provider: IdProvider,
    static_transactions: Vec<Transaction>,
    current_index: usize,
    num_transactions: usize,
    config: Arc<Cli>,
}

impl TransactionGenerator {
    pub fn new(id_provider: IdProvider, num_transactions: usize, config: Arc<Cli>) -> Self {
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
            num_transactions,
            config,
        }
    }

    pub fn next(&mut self) -> Option<Transaction> {
        if self.current_index >= self.num_transactions {
            return None;
        }

        // let transaction = self.static_transactions[self.current_index].clone();
        // self.current_index = self.current_index + 1;
        if thread_rng().next_u32() % 100 < self.config.probability_5_shards {
            let shard_1 = thread_rng().next_u32() % self.config.num_shards;
            let shard_2 = thread_rng().next_u32() % self.config.num_shards;
            let shard_3 = thread_rng().next_u32() % self.config.num_shards;
            let shard_4 = thread_rng().next_u32() % self.config.num_shards;
            let shard_5 = thread_rng().next_u32() % self.config.num_shards;
            let shards = vec![
                Shard(shard_1),
                Shard(shard_2),
                Shard(shard_3),
                Shard(shard_4),
                Shard(shard_5),
            ]
            .into_iter()
            .unique()
            .collect::<Vec<_>>();
            return Some(Transaction {
                id: self.id_provider.next(),
                shards,
                effective_fee: 5,
            });
        }
        if thread_rng().next_u32() % 100 < self.config.probability_2_shards {
            let shard_1 = thread_rng().next_u32() % self.config.num_shards;
            let shard_2 = thread_rng().next_u32() % self.config.num_shards;
            let shards = vec![Shard(shard_1), Shard(shard_2)]
                .into_iter()
                .unique()
                .collect::<Vec<_>>();
            return Some(Transaction {
                id: self.id_provider.next(),
                shards,
                effective_fee: 2,
            });
        }
        if thread_rng().next_u32() % 100 < self.config.probability_3_shards {
            let shard_1 = thread_rng().next_u32() % self.config.num_shards;
            let shard_2 = thread_rng().next_u32() % self.config.num_shards;
            let shard_3 = thread_rng().next_u32() % self.config.num_shards;
            let shards = vec![Shard(shard_1), Shard(shard_2), Shard(shard_3)]
                .into_iter()
                .unique()
                .collect::<Vec<_>>();
            return Some(Transaction {
                id: self.id_provider.next(),
                shards,
                effective_fee: 3,
            });
        }

        if thread_rng().next_u32() % 100 < self.config.probability_4_shards {
            let shard_1 = thread_rng().next_u32() % self.config.num_shards;
            let shard_2 = thread_rng().next_u32() % self.config.num_shards;
            let shard_3 = thread_rng().next_u32() % self.config.num_shards;
            let shard_4 = thread_rng().next_u32() % self.config.num_shards;
            let shards = vec![
                Shard(shard_1),
                Shard(shard_2),
                Shard(shard_3),
                Shard(shard_4),
            ]
            .into_iter()
            .unique()
            .collect::<Vec<_>>();
            return Some(Transaction {
                id: self.id_provider.next(),
                shards,
                effective_fee: 4,
            });
        }

        // otherwise 1 shard
        let shard = thread_rng().next_u32() % self.config.num_shards;
        Some(Transaction {
            id: self.id_provider.next(),
            shards: vec![Shard(shard)],
            effective_fee: 1,
        })
    }
}
