use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use crate::cli::Cli;
use crate::message::Message;
use crate::node_id::NodeId;
use crate::transaction::{Shard, Transaction};
use crate::block::Block;
use crate::committee_manager::CommitteeManager;
use crate::id_provider::IdProvider;
use crate::subscriber::Subscriber;
use itertools::Itertools;
use log::*;
use crate::qc::Qc;

pub struct ValidatorNode {
    pub id: u32,
    pub shard: Shard,
    pub new_tx_mempool: Vec<(u128, Arc<Transaction>)>,
    pub incoming_messages: VecDeque<(u128, Message)>,
    pub hotstuff_round: usize,
    pub index_in_shard: usize,
    pub shard_size: usize,
    pub last_proposed_round: Option<usize>,
    pub config: Arc<Cli>,
    pub blocks: HashMap<u32, Arc<Block>>,
    pub b_leaf: Arc<Block>,
    pub locked_node: Arc<Block>,
    pub high_qc: Arc<Qc>,
    pub last_voted_height: u32,
    id_provider: IdProvider,
    subscriber: Subscriber,
    committee_manager: CommitteeManager,
}


impl ValidatorNode {
        pub fn new(id: u32, shard:Shard, index_in_shard: usize, shard_size: usize, config: Arc<Cli>, genesis: Arc<Block>, id_provider: IdProvider, committee_manager: CommitteeManager) -> Self {
            let mut blocks = HashMap::new();
            blocks.insert(0, genesis.clone());
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
                blocks,
                b_leaf: genesis.clone(),
                locked_node: genesis.clone(),
                high_qc: genesis.justify.clone(),
                id_provider,
                subscriber: Subscriber::new(),
                committee_manager,
                last_voted_height: 0
            }
        }

    pub fn add_transaction(&mut self, transaction: Arc<Transaction>, at_time: u128) {
        if transaction.shards.contains(&self.shard) {
            dbg!("adding to mempool");
            self.new_tx_mempool.push((at_time, transaction));
        } else {
            eprintln!("Transaction {:?} does not belong to shard {:?}", transaction, self.shard)
        }
    }

    pub fn deliver_message(&mut self, message: Message, at_time: u128) {
        self.incoming_messages.push_back((at_time, message));
    }

    pub async fn on_receive_proposal(&mut self, block: Arc<Block>) -> Vec<(u32, Message)> {
        let justify_node = self.blocks.get(&block.justify.block_id).expect("justify parent was missing, should request it");
        let mut result_messages = vec![];
        if block.height > self.last_voted_height && (self.does_extend(&block, &self.locked_node) || justify_node.height > self.locked_node.height) {
            self.last_voted_height = block.height;

            // send vote
            result_messages.push((self.committee_manager.next_leader(self.shard, block.proposed_by).await, Message::Vote{ id: self.id_provider.next(), block_id: block.id, vote_by: self.id} ))
        }

        result_messages
    }

    fn does_extend(&self, block: &Block, ancestor: &Block) -> bool {
        // TODO: check for not only a direct descendant
        block.parent_id == ancestor.id
    }

    pub async fn update(&mut self, current_time: u128) -> Vec<(u32, Message)> {
        // todo: messages per second
        let incoming_messages = self.incoming_messages.drain(..).collect::<Vec<_>>();

        let mut outgoing = vec![];
        for (time, message) in incoming_messages {
            match message {
                Message::Transaction{ tx, ..} => {
                   self.add_transaction(tx, time);

                }
                Message::BlockProposal{ block, ..} => {
                    outgoing.extend( self.on_receive_proposal(block).await);
                },
                Message::Vote {block_id, vote_by,..} => {
                    todo!()
                }
            }
        }

        // on_beat
        if self.is_leader() && (self.last_proposed_round.is_none() || self.last_proposed_round.unwrap() < self.hotstuff_round) {
                self.on_propose().await
        } else {
            outgoing
        }
    }

    fn is_leader(&self) -> bool {
        self.index_in_shard == self.hotstuff_round % self.shard_size
    }


    fn create_leaf(&self, parent_id: u32, transactions: Vec<Arc<Transaction>>, qc: Arc<Qc>, height: u32) -> Arc<Block> {
        Arc::new(Block::new(self.id_provider.next(), parent_id, qc, height, self.id))
    }

    async fn on_propose(&mut self) -> Vec<(u32, Message)> {
        let transactions = self.new_tx_mempool.iter().take(self.config.max_block_size).map(|(_, transaction)| transaction.clone()).collect::<Vec<_>>();
        println!("Node {:?} proposed transactions {:?}", self.id, transactions);


        let high_qc = self.high_qc.clone();
        // if high_qc.view number > generic_qc then self.generic_qc = high_qc

        let mut outgoing = vec![];

        let involved_shards: Vec<Shard> = transactions.iter().map(|transaction| transaction.shards.clone()).flatten().unique().collect();
        let block = self.create_leaf(self.b_leaf.id, transactions, high_qc, self.b_leaf.height + 1);
        self.last_proposed_round = Some(self.hotstuff_round);

        // send to all nodes.
        dbg!(involved_shards.clone());
        for shard in involved_shards {
            if shard == self.shard {
                // skip our own shard, because everyone in this committee will get it
                continue;
            }
            let committee = self.committee_manager.get_committee(shard).await;
            dbg!(committee.clone());
            for node_id in committee {
                outgoing.push((node_id, Message::BlockProposal{ id: self.id_provider.next(),  block: block.clone()}));
            }

        }

        for local in self.committee_manager.get_committee(self.shard).await {
            outgoing.push((local, Message::BlockProposal{ id: self.id_provider.next(),  block: block.clone()}));
        }

        // saved when receiving
        // self.blocks.push(block.clone());
        self.b_leaf = block.clone();
        outgoing
    }
}
