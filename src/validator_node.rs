use std::sync::Arc;
use std::collections::VecDeque;
use crate::cli::Cli;
use crate::message::Message;
use crate::node_id::NodeId;
use crate::transaction::{Shard, Transaction};
use crate::block::Block;
use crate::committee_manager::CommitteeManager;
use crate::id_provider::IdProvider;
use crate::subscriber::Subscriber;
use itertools::Itertools;

pub struct ValidatorNode {
    pub id: u32,
    pub shard: usize,
    pub new_tx_mempool: Vec<(u128, Transaction)>,
    pub incoming_messages: VecDeque<(u128, Message)>,
    pub hotstuff_round: usize,
    pub index_in_shard: usize,
    pub shard_size: usize,
    pub last_proposed_round: Option<usize>,
    pub config: Arc<Cli>,
    pub blocks: Vec<Arc<Block>>,
    pub tip: Arc<Block>,
    id_provider: IdProvider,
    subscriber: Subscriber,
    committee_manager: CommitteeManager,
}


impl ValidatorNode {
        pub fn new(id: u32, shard:usize, index_in_shard: usize, shard_size: usize, config: Arc<Cli>, genesis: Arc<Block>, id_provider: IdProvider, committee_manager: CommitteeManager) -> Self {
            Self {
                id,
                shard,
                new_tx_mempool: Vec::new(),
                incoming_messages: VecDeque::new(),
                hotstuff_round: 0,
                index_in_shard,
                shard_size,
                last_proposed_round: None,
                config,
                blocks: vec![genesis.clone()],
                tip: genesis,
                id_provider,
                subscriber: Subscriber::new(),
                committee_manager
            }
        }

    pub fn add_transaction(&mut self, transaction: Transaction, at_time: u128) {
        dbg!("adding to mempool");
        self.new_tx_mempool.push((at_time, transaction));
    }

    pub fn deliver_message(&mut self, message: Message, at_time: u128) {
        self.incoming_messages.push_back((at_time, message));
    }

    pub fn update(&mut self, current_time: u128) -> Vec<(u32, Message)> {
        // todo: messages per second
        let incoming_messages = self.incoming_messages.drain(..).collect::<Vec<_>>();
        for (time, message) in incoming_messages {
            match message {
                Message::Transaction(_, transaction) => {
                   self.add_transaction(transaction, time);

                }
            }
        }

        let mut outgoing = vec![];
        if self.is_leader() && (self.last_proposed_round.is_none() || self.last_proposed_round.unwrap() < self.hotstuff_round) {
            let transactions = self.new_tx_mempool.iter().take(self.config.max_block_size).map(|(_, transaction)| transaction).collect::<Vec<_>>();
            println!("Node {:?} proposed transactions {:?}", self.id, transactions);

            let block = Arc::new(Block::new(self.id_provider.next(), self.tip.id));
            self.last_proposed_round = Some(self.hotstuff_round);

            // send to all nodes.
            let involved_shards: Vec<Shard> = transactions.iter().map(|transaction| transaction.shards.clone()).flatten().unique().collect();
            dbg!(involved_shards.clone());

        }

        outgoing

    }

    fn is_leader(&self) -> bool {
        self.index_in_shard == self.hotstuff_round % self.shard_size
    }
}
