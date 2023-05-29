use std::sync::Arc;
use std::collections::VecDeque;
use crate::cli::Cli;
use crate::message::Message;
use crate::node_id::NodeId;
use crate::transaction::Transaction;

pub struct ValidatorNode {
    id: NodeId,
    shard: usize,
    new_tx_mempool: Vec<(u128, Transaction)>,
    incoming_messages: VecDeque<(u128, Message)>,
    hotstuff_round: usize,
    index_in_shard: usize,
    shard_size: usize,
    last_proposed_round: Option<usize>,
    config: Arc<Cli>
}

impl ValidatorNode {
        fn new(id: NodeId, shard:usize, index_in_shard: usize, shard_size: usize, config: Arc<Cli>) -> Self {
            Self {
                id,
                shard,
                new_tx_mempool: Vec::new(),
                incoming_messages: VecDeque::new(),
                hotstuff_round: 0,
                index_in_shard,
                shard_size,
                last_proposed_round: None,
                config
            }
        }

    fn add_transaction(&mut self, transaction: Transaction, at_time: u128) {
        dbg!("adding to mempool");
        self.new_tx_mempool.push((at_time, transaction));
    }

    fn deliver_message(&mut self, message: Message, at_time: u128) {
        self.incoming_messages.push_back((at_time, message));
    }

    fn update(&mut self, current_time: u128) {
        // todo: messages per second
        let incoming_messages = self.incoming_messages.drain(..).collect::<Vec<_>>();
        for (time, message) in incoming_messages {
            match message {
                Message::Transaction(_, transaction) => {
                   self.add_transaction(transaction, time);

                }
            }
        }

        if self.is_leader() && (self.last_proposed_round.is_none() || self.last_proposed_round.unwrap() < self.hotstuff_round) {
            self.last_proposed_round = Some(self.hotstuff_round);
            let transactions = self.new_tx_mempool.iter().take(self.config.max_block_size).map(|(_, transaction)| transaction).collect::<Vec<_>>();
            println!("Node {:?} proposed transactions {:?}", self.id, transactions);
        }
    }

    fn is_leader(&self) -> bool {
        self.index_in_shard == self.hotstuff_round % self.shard_size
    }
}
