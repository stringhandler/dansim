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
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;

#[derive(Debug)]
pub struct ValidatorNode {
    pub id: u32,
    pub shard: Shard,
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
    // Mempools
    pub new_tx_mempool: BinaryHeap<Arc<Transaction>>,
    pub waiting_prepared_mempool: HashMap<u32, (Arc<Transaction>, HashMap<Shard, Arc<Block>>)>,
    pub ready_prepared_mempool: BinaryHeap<Arc<Transaction>>,
    pub waiting_pre_committed_mempool: HashMap<u32, (Arc<Transaction>, HashMap<Shard, Arc<Block>>)>,
    pub ready_pre_committed_mempool: BinaryHeap<Arc<Transaction>>,
    // TODO: I don't think I need a committed mempool....
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
            new_tx_mempool: BinaryHeap::new(),
            waiting_prepared_mempool: HashMap::new(),
            ready_prepared_mempool: BinaryHeap::new(),
            waiting_pre_committed_mempool: HashMap::new(),
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
            ready_pre_committed_mempool: BinaryHeap::new(),
        }
    }

    pub fn print_stats(&self) {
        println!(
            "VN {} stats: shard:{} current_leader: {} height: {} b_leaf: {}, b_exec: {}, locked_node: {}, high_qc: {}, new_tx: {} prepare_w: {} prep_r:{}, precomm_w: {}, precom_r:{}",
            self.id,
            self.shard.0,
            self.current_leader,
            self.current_height,
            self.b_leaf.height,
            self.b_exec.height,
            self.locked_node.height,
            self.high_qc.block_height,
            self.new_tx_mempool.len(),
            self.waiting_prepared_mempool.len(),
            self.ready_prepared_mempool.len(),
            self.waiting_pre_committed_mempool.len(),
            self.ready_pre_committed_mempool.len()
        );

        // for tx in self.new_tx_mempool.iter() {
        //     println!("new tx: {:?}", tx);
        // }
        // for tx in self.waiting_prepared_mempool.iter() {
        //     println!("waiting prep tx: {:?}", tx);
        // }
        // for tx in self.ready_prepared_mempool.iter() {
        //     println!("ready prep tx: {:?}", tx);
        // }
        // for tx in self.waiting_pre_committed_mempool.iter() {
        //     println!("waiting precom tx: {:?}", tx);
        // }
    }

    pub fn add_transaction(&mut self, transaction: Arc<Transaction>, at_time: u128) {
        if transaction.shards.contains(&self.shard) {
            self.new_tx_mempool.push(transaction);
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
        let mut result_messages = vec![];
        if block.shard == self.shard {
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

                self.update_blocks(block, current_time).await;
            }
        } else {
            dbg!("Foreignly");
            // Foreign committee block
            for tx in &block.prepare_txs {
                // check if we have a prepare waiting
                let mut must_remove = false;
                if let Some((tx, shard_nodes)) = self.waiting_prepared_mempool.get_mut(&tx.id) {
                    if !shard_nodes.contains_key(&block.shard) {
                        dbg!("Added vote");
                        shard_nodes.insert(block.shard, block.clone());
                        // check if we can move it to ready
                        if shard_nodes.len() == tx.shards.len() {
                            self.ready_prepared_mempool.push(tx.clone());
                            must_remove = true;
                            // self.waiting_prepared_mempool.remove(&tx.id);
                            // TODO: attach all nodes to the tx
                            self.subscriber.on_transaction_moved_to_prepare_ready(
                                tx.id,
                                self.id,
                                current_time,
                                block.id,
                            );
                        }
                    }
                } else {
                    // add it... it will still need to be prepared locally, so leave it in the new_tx pool as well.
                    self.waiting_prepared_mempool.insert(
                        tx.id,
                        (
                            tx.clone(),
                            [(block.shard, block.clone())].iter().cloned().collect(),
                        ),
                    );
                }

                if must_remove {
                    self.waiting_prepared_mempool.remove(&tx.id);
                }
            }
        }

        result_messages
    }

    async fn update_blocks(&mut self, block: Arc<Block>, current_time: u128) {
        self.current_height = block.height;

        let b_dash_dash = self
            .blocks
            .get(&block.justify.block_id)
            .expect("justify parent was missing, should request it")
            .clone();

        // TODO: it's not clear whether we should
        // commit foreign consensus blocks here or wait for a 3 chain locally.
        // for now, I'm going to assume it's fine to commit them here
        // ======= Prepare ======

        self.apply_qc(&block.justify, current_time, &b_dash_dash)
            .await;

        /// other hotstuff
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

    async fn apply_qc(&mut self, qc: &Arc<Qc>, current_time: u128, justified_node: &Arc<Block>) {
        for tx in &justified_node.prepare_txs {
            // local cerb
            if tx.shards.len() == 1 && tx.shards.contains(&self.shard) {
                self.ready_prepared_mempool.push(tx.clone());
                self.subscriber
                    .on_transaction_prepared_ready(tx.id, self.shard, qc.clone(), current_time)
                    .await;
            } else {
                // remote cerb
                let entry = self
                    .waiting_prepared_mempool
                    .entry(tx.id)
                    .or_insert_with(|| (tx.clone(), HashMap::new()));
                entry.1.insert(self.shard, justified_node.clone());
                // move to ready if we have all the shards
                if entry.1.len() == tx.shards.len() {
                    self.ready_prepared_mempool.push(tx.clone());
                    self.waiting_prepared_mempool.remove(&tx.id);
                    self.subscriber
                        .on_transaction_prepared_ready(tx.id, self.shard, qc.clone(), current_time)
                        .await;
                } else {
                    self.subscriber
                        .on_transaction_prepared_waiting(
                            tx.id,
                            self.shard,
                            qc.clone(),
                            current_time,
                        )
                        .await;
                }
            }
        }

        //TODO: should apply all the txs in the previous blocks
        let new_tx: Vec<Arc<Transaction>> = self.new_tx_mempool.drain().collect();
        for tx in new_tx {
            if !justified_node.prepare_txs.contains(&tx) {
                self.new_tx_mempool.push(tx);
            }
        }

        // ==== precomitted =======
        for tx in &justified_node.precommit_txs {
            // local cerb
            if tx.shards.len() == 1 && tx.shards.contains(&self.shard) {
                self.ready_pre_committed_mempool.push(tx.clone());
                self.subscriber
                    .on_transaction_precommit_ready(tx.id, self.shard, qc.clone(), current_time)
                    .await;
            } else {
                // remote cerb

                // self.waiting_pre_committed_mempool
                //     .push((current_time, tx.clone()));
                // self.subscriber
                //     .on_transaction_precommit_waiting(tx.id, self.shard, qc.clone(), current_time)
                //     .await;
                let entry = self
                    .waiting_pre_committed_mempool
                    .entry(tx.id)
                    .or_insert_with(|| (tx.clone(), HashMap::new()));
                entry.1.insert(self.shard, justified_node.clone());
                // move to ready if we have all the shards
                if entry.1.len() == tx.shards.len() {
                    self.ready_pre_committed_mempool.push(tx.clone());
                    self.waiting_pre_committed_mempool.remove(&tx.id);
                    self.subscriber
                        .on_transaction_precommit_ready(tx.id, self.shard, qc.clone(), current_time)
                        .await;
                } else {
                    self.subscriber
                        .on_transaction_precommit_waiting(
                            tx.id,
                            self.shard,
                            qc.clone(),
                            current_time,
                        )
                        .await;
                }
            }
        }

        let preps: Vec<Arc<Transaction>> = self.ready_prepared_mempool.drain().collect();
        for tx in preps {
            if !justified_node.precommit_txs.contains(&tx) {
                self.ready_prepared_mempool.push(tx);
            }
        }

        for pre_comms in &justified_node.precommit_txs {
            if self.waiting_prepared_mempool.contains_key(&pre_comms.id) {
                self.waiting_prepared_mempool.remove(&pre_comms.id);
            }
        }

        // let waiting: Vec<(u128, Arc<Transaction>)> =
        //     self.waiting_prepared_mempool.drain(..).collect();
        // for (time, tx) in waiting {
        //     if !justified_node.precommit_txs.contains(&tx) {
        //         self.waiting_prepared_mempool.push((time, tx));
        //     }
        // }

        // ====== Committed ...... Can save to DB ======

        for tx in &justified_node.commit_txs {
            println!("APPLIED TX: {:?}", tx);
            self.subscriber
                .on_transaction_committed(tx.id, self.shard, qc.clone(), current_time)
                .await;
        }

        let preps: Vec<Arc<Transaction>> = self.ready_pre_committed_mempool.drain().collect();
        for tx in preps {
            if !justified_node.commit_txs.contains(&tx) {
                self.ready_pre_committed_mempool.push(tx);
            }
        }

        for pre_comms in &justified_node.commit_txs {
            if self
                .waiting_pre_committed_mempool
                .contains_key(&pre_comms.id)
            {
                self.waiting_pre_committed_mempool.remove(&pre_comms.id);
            }
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
                    dbg!("Got a vote");
                    dbg!(self.id);
                    has_new_qc_or_can_propose |= self
                        .on_receive_vote(*block_id, *block_height, *vote_by, current_time)
                        .await;
                    dbg!(has_new_qc_or_can_propose);
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
            dbg!("Has new qc, checking if I am the leader");
            self.print_stats();
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
                if self.is_leader().await {
                    outgoing.extend(self.on_propose(current_time).await);
                }
            }

            // Send on start up to let the leader know we are here
            if self.current_height == 0
                || self.time_last_proposal_received + self.config.delta.as_millis() <= current_time
            {
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
        // Exclude the first set up
        if self.current_height != 0 {
            self.subscriber.on_leader_failure(self.id).await;
        }
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
        // Send the first time only (== instead of >=)

        if votes.len() == shard_size - ((shard_size - 1) / 3) {
            let qc = Arc::new(Qc::new(
                self.id_provider.next(),
                block_id,
                block_height,
                votes.clone(),
            ));
            self.subscriber
                .on_qc_created(qc.id, current_time, qc.block_id)
                .await;
            // Apply the node so that we can propose a new block using the updated mempools
            let qc_block = self
                .blocks
                .get(&qc.block_id)
                .expect("block missing")
                .clone();
            self.apply_qc(&qc, current_time, &qc_block).await;
            self.update_high_qc(qc);
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
        &mut self,
        parent_id: u32,
        qc: Arc<Qc>,
        height: u32,
        current_time: u128,
    ) -> Arc<Block> {
        let mut prepare_txs = vec![];
        let mut precommit_txs = vec![];
        let mut commit_txs = vec![];
        // The transactions in the QC have not yet been moved to their new pools, so we need to make sure we don't
        // add them again here.

        let qc_block = self.blocks.get(&qc.block_id).unwrap();

        loop {
            if prepare_txs.len() < self.config.max_tx_per_step_per_block {
                if let Some(transaction) = self.new_tx_mempool.pop() {
                    // if !qc_block.prepare_txs.contains(&transaction) {
                    prepare_txs.push(transaction);
                    // }
                }
            }
            if precommit_txs.len() < self.config.max_tx_per_step_per_block {
                if let Some(tx) = self.ready_prepared_mempool.pop() {
                    // if !qc_block.precommit_txs.contains(&tx) {
                    precommit_txs.push(tx);
                    // }
                }
            }
            if commit_txs.len() < self.config.max_tx_per_step_per_block {
                if let Some(tx) = self.ready_pre_committed_mempool.pop() {
                    // if !qc_block.commit_txs.contains(&tx) {
                    commit_txs.push(tx);
                    // }
                }
            }

            if (prepare_txs.len() == self.config.max_tx_per_step_per_block
                || self.new_tx_mempool.is_empty())
                && (precommit_txs.len() == self.config.max_tx_per_step_per_block
                    || self.ready_prepared_mempool.is_empty())
                && (commit_txs.len() == self.config.max_tx_per_step_per_block
                    || self.ready_pre_committed_mempool.is_empty())
            {
                break;
            }
        }
        // let prepare_txs = self.new_tx_mempool.pop()
        // let precommit_txs = self.ready_prepare_mempool.drain(..self.config.max_tx_per_step_per_block).collect();
        let block = Arc::new(Block::new(
            self.id_provider.next(),
            parent_id,
            self.shard,
            qc,
            height,
            self.id,
            prepare_txs,
            precommit_txs,
            commit_txs,
        ));
        self.subscriber
            .on_create_leaf(block.clone(), current_time)
            .await;
        block
    }

    async fn on_propose(&mut self, current_time: u128) -> Vec<(u32, Message)> {
        let high_qc = self.high_qc.clone();
        // if high_qc.view number > generic_qc then self.generic_qc = high_qc

        let mut outgoing = vec![];

        let block = self
            .create_leaf(
                self.b_leaf.id,
                high_qc,
                self.b_leaf.height + 1,
                current_time,
            )
            .await;
        self.last_proposed_round = Some(self.current_height);

        let involved_shards = block.involved_shards();
        // send to all nodes.
        for shard in involved_shards {
            if shard == self.shard {
                continue;
            }
            let committee = self.committee_manager.get_committee(shard).await;
            for node_id in committee {
                outgoing.push((
                    node_id,
                    Message::BlockProposal {
                        id: self.id_provider.next(),
                        block: block.clone(),
                    },
                ));
            }
        }

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
