use crate::block::Block;
use crate::cli::Cli;
use crate::committee_manager::CommitteeManager;
use crate::id_provider::IdProvider;
use crate::message::Message;
use crate::node_id::NodeId;
use crate::qc::Qc;
use crate::subscriber::Subscriber;
use crate::transaction::{Shard, Transaction};
use itertools::Itertools;
use log::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(Debug)]
pub struct ValidatorNode {
    pub id: u32,
    pub shard: Shard,
    pub new_tx_mempool: Vec<(u128, Arc<Transaction>)>,
    pub incoming_messages: VecDeque<(u128, Message)>,
    pub current_height: u32,
    pub current_leader: u32,
    pub last_proposed_round: Option<u32>,
    pub config: Arc<Cli>,
    pub blocks: HashMap<u32, Arc<Block>>,
    pub b_leaf: Arc<Block>,
    pub locked_node: Arc<Block>,
    pub high_qc: Arc<Qc>,
    pub last_voted_height: u32,
    id_provider: IdProvider,
    subscriber: Subscriber,
    committee_manager: CommitteeManager,
    votes: HashMap<u32, Vec<u32>>,
    time_last_proposal_received: u128,
    b_exec: Arc<Block>,
    snoozed_messages: HashMap<u32, Vec<(u128, Message)>>,
    pub base_latency: u128,
    new_view_votes: HashMap<u32, Vec<u32>>,
}

impl ValidatorNode {
    pub fn new(
        id: u32,
        shard: Shard,
        config: Arc<Cli>,
        genesis: Arc<Block>,
        id_provider: IdProvider,
        committee_manager: CommitteeManager,
        subscriber: Subscriber,
        base_latency: u128,
    ) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(0, genesis.clone());
        Self {
            id,
            shard,
            new_tx_mempool: Vec::new(),
            incoming_messages: VecDeque::new(),
            current_height: 0,
            current_leader: 0,
            last_proposed_round: None,
            config,
            blocks,
            b_leaf: genesis.clone(),
            locked_node: genesis.clone(),
            high_qc: genesis.justify.clone(),
            id_provider,
            subscriber,
            committee_manager,
            last_voted_height: 0,
            votes: HashMap::new(),
            time_last_proposal_received: 0,
            b_exec: genesis.clone(),
            snoozed_messages: HashMap::new(),
            base_latency,
            new_view_votes: HashMap::new(),
        }
    }

    pub fn print_stats(&self) {
        println!(
            "VN {} stats: shard:{} current_leader: {} height: {} b_leaf: {}, b_exec: {}, locked_node: {}, high_qc: {}",
            self.id,
            self.shard.0,
            self.current_leader,
            self.current_height,
            self.b_leaf.height,
            self.b_exec.height,
            self.locked_node.height,
            self.high_qc.block_height
        );
    }

    pub fn add_transaction(&mut self, transaction: Arc<Transaction>, at_time: u128) {
        if transaction.shards.contains(&self.shard) {
            dbg!("adding to mempool");
            self.new_tx_mempool.push((at_time, transaction));
        } else {
            eprintln!(
                "Transaction {:?} does not belong to shard {:?}",
                transaction, self.shard
            )
        }
    }

    pub fn deliver_message(&mut self, message: Message, at_time: u128) {
        self.incoming_messages.push_back((at_time, message));
    }

    pub async fn on_receive_proposal(
        &mut self,
        block: Arc<Block>,
        current_time: u128,
        original_message: (u128, Message),
    ) -> Vec<(u32, Message)> {
        let justify_node = match self.blocks.get(&block.justify.block_id) {
            Some(node) => node.clone(),
            None => {
                self.subscriber.on_request_block(self.id).await;

                self.snoozed_messages
                    .entry(block.justify.block_id)
                    .or_insert_with(Vec::new)
                    .push(original_message);
                // TODO: Maybe we should ask many nodes for the block
                // TODO: Maybe we should provide a few blocks with the proposal, depending on space
                return vec![(
                    block.proposed_by,
                    Message::RequestBlock {
                        id: self.id_provider.next(),
                        block_id: block.justify.block_id,
                        request_by: self.id,
                    },
                )];
            }
        };

        let mut result_messages = vec![];
        if block.height > self.last_voted_height
            && (self.does_extend(&block, &self.locked_node)
                || justify_node.height > self.locked_node.height)
        {
            self.last_voted_height = block.height;

            // send vote
            result_messages.push((
                self.committee_manager
                    .next_leader(self.shard, block.proposed_by)
                    .await,
                Message::Vote {
                    id: self.id_provider.next(),
                    block_id: block.id,
                    block_height: block.height,
                    vote_by: self.id,
                },
            ));

            self.update_blocks(block);
        }

        result_messages
    }

    fn update_blocks(&mut self, block: Arc<Block>) {
        self.current_height = block.height;
        let b_dash_dash = self
            .blocks
            .get(&block.justify.block_id)
            .expect("justify parent was missing, should request it")
            .clone();

        let b_dash = self
            .blocks
            .get(&b_dash_dash.justify.block_id)
            .expect("justify parent was missing, should request it")
            .clone();

        let b = self
            .blocks
            .get(&b_dash.justify.block_id)
            .expect("justify parent was missing, should request it")
            .clone();

        self.update_high_qc(block.justify.clone());
        if b_dash.height > self.locked_node.height {
            self.locked_node = b_dash.clone();
        }
        // Should we not just commit b?
        // This would exclude dummy blocks from being executed until there is a 3 chain
        if b_dash_dash.parent_id == b_dash.id && b_dash.parent_id == b.id {
            self.on_commit(b.clone());
            self.b_exec = b.clone();
        }
    }

    fn on_commit(&mut self, block: Arc<Block>) {
        if self.b_exec.height < block.height {
            let parent = self
                .blocks
                .get(&block.parent_id)
                .expect("justify parent was missing, should request it");
            self.on_commit(parent.clone());
            self.execute(&block);
        }
    }

    fn execute(&mut self, block: &Block) {
        // for tx in &block.transactions {
        //     self.subscriber.on_execute(self.id, tx.clone());
        // }
    }

    fn does_extend(&self, block: &Block, ancestor: &Block) -> bool {
        // TODO: check for not only a direct descendant
        block.parent_id == ancestor.id
    }

    pub async fn update(&mut self, current_time: u128) -> Vec<(u32, Message)> {
        // todo: messages per second
        let incoming_messages = self.incoming_messages.drain(..).collect::<Vec<_>>();

        let mut outgoing = vec![];
        let mut has_new_qc_or_can_propose = false;
        for (time, message) in incoming_messages {
            match &message {
                Message::Transaction { tx, .. } => {
                    self.add_transaction(tx.clone(), time);
                }
                Message::BlockProposal { block, .. } => {
                    self.time_last_proposal_received = current_time;
                    if !self.blocks.contains_key(&block.id) {
                        self.blocks.insert(block.id, block.clone());
                    } else {
                        dbg!("Got a duplicate block proposal");
                    }
                    outgoing.extend(
                        self.on_receive_proposal(block.clone(), current_time, (time, message))
                            .await,
                    );
                }
                Message::NewView {
                    height,
                    high_qc,
                    from,
                    ..
                } => {
                    self.update_high_qc(high_qc.clone());
                    if *height >= self.current_height {
                        // self.current_height = *height;
                        // self.locked_node = self.high_qc.clone();
                        // self.on_commit(self.high_qc.clone());
                        self.new_view_votes
                            .entry(*height)
                            .or_insert_with(Vec::new)
                            .push(*from);
                        let n = self.committee_manager.get_committee(self.shard).await.len();
                        if self.new_view_votes.get(height).unwrap().len() > n - ((n - 1) / 3) {
                            // must propose.
                            has_new_qc_or_can_propose = true;
                        }
                    }
                }
                Message::Vote {
                    block_id,
                    block_height,
                    vote_by,
                    ..
                } => {
                    has_new_qc_or_can_propose = self
                        .on_receive_vote(*block_id, *block_height, *vote_by, current_time)
                        .await;
                }
                Message::RequestBlock {
                    block_id,
                    request_by,
                    ..
                } => {
                    if let Some(block) = self.blocks.get(&block_id) {
                        outgoing.push((
                            *request_by,
                            Message::RequestBlockResponse {
                                id: self.id_provider.next(),
                                block: block.clone(),
                            },
                        ));
                    }
                }
                Message::RequestBlockResponse { block, .. } => {
                    if !self.blocks.contains_key(&block.id) {
                        dbg!("Got a block response");
                        self.blocks.insert(block.id, block.clone());
                        // unsnooze messages
                        let messages = self.snoozed_messages.remove(&block.id).unwrap_or_default();
                        for (time, message) in messages {
                            // self.incoming_messages.push_back((time, message));
                            outgoing.push((self.id, message));
                        }
                    }
                }
            }
        }

        // on_beat
        if has_new_qc_or_can_propose
        // ||
        {
            dbg!("on beat");
            if self.is_leader().await
            // && (self.last_proposed_round.is_none()
            //     || self.last_proposed_round.unwrap() < self.hotstuff_round)
            {
                outgoing.extend(self.on_propose(current_time).await);
            }
        } else {
            // propose early to avoid failure
            if self.time_last_proposal_received + self.config.delta.as_millis() / 2 <= current_time
            {
                // if leader, just propose
                dbg!("on beat");
                if self.is_leader().await {
                    outgoing.extend(self.on_propose(current_time).await);
                }
            }

            if self.time_last_proposal_received + self.config.delta.as_millis() <= current_time {
                // if leader, just propose
                {
                    outgoing.extend(self.on_next_sync_view().await);
                    self.time_last_proposal_received = current_time;
                }
            }
        }
        outgoing
    }

    async fn on_next_sync_view(&mut self) -> Vec<(u32, Message)> {
        // self.hotstuff_round += 1;
        self.subscriber.on_leader_failure(self.id).await;
        dbg!("on next sync view");
        let next_leader = self
            .committee_manager
            .next_leader(self.shard, self.current_leader)
            .await;
        self.current_leader = next_leader;
        self.current_height += 1;
        vec![(
            next_leader,
            Message::NewView {
                id: self.id_provider.next(),
                height: self.current_height,
                high_qc: self.high_qc.clone(),
                from: self.id,
            },
        )]
    }

    async fn on_receive_vote(
        &mut self,
        block_id: u32,
        block_height: u32,
        vote_by: u32,
        current_time: u128,
    ) -> bool {
        if block_height < self.current_height {
            eprintln!("Received a vote for a block that is too old");
            return false;
        }
        let votes = self.votes.entry(block_id).or_insert_with(Vec::new);
        if !votes.contains(&vote_by) {
            votes.push(vote_by);
            self.subscriber
                .on_vote(vote_by, block_id, current_time)
                .await;
        }

        let votes = votes.clone();
        let shard_size = self.committee_manager.get_committee(self.shard).await.len();
        if votes.len() >= shard_size - ((shard_size - 1) / 3) {
            self.update_high_qc(Arc::new(Qc::new(
                self.id_provider.next(),
                block_id,
                block_height,
                votes.clone(),
            )));
            true
        } else {
            false
        }
    }

    fn update_high_qc(&mut self, qc: Arc<Qc>) {
        if qc.block_height > self.high_qc.block_height {
            self.b_leaf = self
                .blocks
                .get(&qc.block_id)
                .expect("justify parent was missing, should request it")
                .clone();
            self.high_qc = qc;
        }
    }
    async fn is_leader(&self) -> bool {
        let next_leader = self
            .committee_manager
            .next_leader(self.shard, self.b_leaf.proposed_by)
            .await;

        next_leader == self.id
    }

    async fn create_leaf(
        &self,
        parent_id: u32,
        transactions: Vec<Arc<Transaction>>,
        qc: Arc<Qc>,
        height: u32,
        current_time: u128,
    ) -> Arc<Block> {
        let block = Arc::new(Block::new(
            self.id_provider.next(),
            parent_id,
            qc,
            height,
            self.id,
        ));
        self.subscriber
            .on_create_leaf(block.clone(), current_time)
            .await;
        block
    }

    async fn on_propose(&mut self, current_time: u128) -> Vec<(u32, Message)> {
        let transactions = self
            .new_tx_mempool
            .iter()
            .take(self.config.max_block_size)
            .map(|(_, transaction)| transaction.clone())
            .collect::<Vec<_>>();
        println!(
            "Node {:?} proposed transactions {:?}",
            self.id, transactions
        );

        let high_qc = self.high_qc.clone();
        // if high_qc.view number > generic_qc then self.generic_qc = high_qc

        let mut outgoing = vec![];

        let involved_shards: Vec<Shard> = transactions
            .iter()
            .map(|transaction| transaction.shards.clone())
            .flatten()
            .unique()
            .collect();
        let block = self
            .create_leaf(
                self.b_leaf.id,
                transactions,
                high_qc,
                self.b_leaf.height + 1,
                current_time,
            )
            .await;
        self.last_proposed_round = Some(self.current_height);

        // TODO: Send to other committees
        // send to all nodes.
        // for shard in involved_shards {
        //     if shard == self.shard {
        //         skip our own shard, because everyone in this committee will get it
        // continue;
        // }
        // let committee = self.committee_manager.get_committee(shard).await;
        // dbg!(committee.clone());
        // for node_id in committee {
        //     outgoing.push((
        //         node_id,
        //         Message::BlockProposal {
        //             id: self.id_provider.next(),
        //             block: block.clone(),
        //         },
        //     ));
        // }
        // }

        for local in self.committee_manager.get_committee(self.shard).await {
            outgoing.push((
                local,
                Message::BlockProposal {
                    id: self.id_provider.next(),
                    block: block.clone(),
                },
            ));
        }

        // saved when receiving
        // self.blocks.push(block.clone());
        self.b_leaf = block.clone();
        outgoing
    }
}
