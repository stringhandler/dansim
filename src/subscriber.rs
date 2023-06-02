use crate::block::Block;
use crate::qc::Qc;
use crate::transaction::{Shard, Transaction};
use neo4rs::query;
use neo4rs::Graph;
use neo4rs::Node;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Stats {
    pub leaves_created: usize,
    pub request_block: usize,
    pub leader_failures: usize,
}
#[derive(Clone)]
pub struct Subscriber {
    client: Arc<Graph>,
    stats: Arc<RwLock<HashMap<u32, Stats>>>,
}

impl Subscriber {
    pub async fn on_create_leaf(&self, block: Arc<Block>, time: u128) {
        let mut res = self.client.execute(query("CREATE (b: Block { id: $id, parent_id: $parent_id, height: $height, proposed_by: $proposed_by, t: $time })")
            .param("id", block.id)
            .param("time", time as u32)
            .param("parent_id", block.parent_id)
            .param("height", block.height)
            .param("proposed_by", block.proposed_by)).await.expect("Failed to create block");
        while let Ok(Some(row)) = res.next().await {
            // println!("leaf created");
        }

        let mut res = self.client.execute(query("MATCH (b: Block {id: $id}), (v: VN { id: $vn_id}) CREATE (v)-[n:PROPOSES { t:  $time}]->(b) RETURN n")
            .param("id", block.id)
            .param("time", time as u32)
            .param("vn_id", format!("node_{}", block.proposed_by))).await.expect("Failed to create block");
        while let Ok(Some(row)) = res.next().await {
            // println!("rel created");
        }

        let mut lock = self.stats.write().await;
        lock.entry(block.proposed_by)
            .or_insert_with(|| Stats::default())
            .leaves_created += 1;

        self.client.execute(query("MATCH (b: Block {id: $id}), (q: Qc {id: $qc_id}) CREATE (b)-[n:JUSTIFY]->(q) RETURN n")
            .param("id", block.id)
            .param("qc_id",  block.justify.id)).await.expect("Failed to create qc rel").next().await.expect("failed to create qc rel");
    }

    pub async fn on_vote(&self, vn_id: u32, block_id: u32, t: u128) {
        let mut res = self.client.execute(query("MATCH (b: Block {id: $id}), (v: VN { id: $vn_id}) CREATE (v)-[n:VOTES { t:  $t}]->(b) RETURN n")
            .param("id", block_id)
        .param("t", t as u32)
            .param("vn_id", format!("node_{}", vn_id))).await.expect("Failed to create block");

        while let Ok(Some(row)) = res.next().await {
            // println!("vote created");
        }
    }

    pub async fn create_shard(&self, shard_id: u32) {
        let mut res = self
            .client
            .execute(
                query("CREATE (s: Shard { id: $id })").param("id", format!("shard_{}", shard_id)),
            )
            .await
            .expect("Failed to create shard");
        while let Ok(Some(row)) = res.next().await {
            // println!("shard created");
        }
    }

    pub async fn create_vn(&self, vn_id: u32, shard_id: Shard, latency: u128) {
        let mut res = self
            .client
            .execute(
                query("CREATE (vn: VN:Node { id: $id, latency: $latency })")
                    .param("id", format!("node_{}", vn_id))
                    .param("latency", latency as u32),
            )
            .await
            .expect("Failed to create vn");
        while let Ok(Some(row)) = res.next().await {
            // println!("vn created");
        }
        let mut res = self.client
            .execute(
                query("MATCH (vn: VN { id: $vn_id}), (s:Shard { id: $id })  CREATE (vn)-[n:BELONGS_TO]->(s) RETURN n")
                    .param("id", format!("shard_{}", shard_id.0))
                    .param("vn_id", format!("node_{}", vn_id)),
            )
            .await
            .expect("Could not link to shard");

        while let Ok(Some(row)) = res.next().await {
            // println!("rel created");
        }
    }

    pub async fn create_indexer(&self, id: u32) {
        let mut res = self
            .client
            .execute(
                query("CREATE (in: Indexer:Node { id: $id })").param("id", format!("node_{}", id)),
            )
            .await
            .expect("Failed to create indexer");
        while let Ok(Some(row)) = res.next().await {
            // println!("indexer created");
        }
    }

    pub async fn on_transaction_queued(&self, tx_id: u32, t: u128, tx: &Transaction) {
        self.client
            .execute(
                query("CREATE (t: Transaction {id: $tx_id, t: $t, shards: $shards})")
                    .param("tx_id", tx_id)
                    .param("t", t as u32)
                    .param(
                        "shards",
                        tx.shards
                            .iter()
                            .map(|s| s.0.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                    ),
            )
            .await
            .expect("Failed to create block")
            .next()
            .await
            .expect("Failed to create tx");
    }

    pub async fn on_transaction_moved_to_prepare_ready(
        &self,
        tx_id: u32,
        in_vn: u32,
        t: u128,
        in_block: u32,
    ) {
        self.client.execute(query("MATCH (t: Transaction {id: $tx_id}), (b: Block {id: $in_block}) CREATE (t)-[n: MOVES_TO_PREPARE_READY {t:  $t}]->(b) RETURN n").param("tx_id", tx_id)
            .param("t", t as u32)
            .param("in_block", in_block)).await.expect("Failed to create tx move to ready").next().await.expect("Failed to create rel");
    }

    pub async fn on_qc_created(&self, qc_id: u32, t: u128, block_id: u32) {
        self.client
            .execute(
                query("MERGE (qc: Qc {id: $qc_id, t: $t})")
                    .param("qc_id", qc_id)
                    .param("t", t as u32),
            )
            .await
            .expect("Failed to create qc")
            .next()
            .await
            .expect("Failed to create qc");

        self.client.execute(query("MATCH (b: Block {id: $id}), (qc: Qc { id: $qc_id}) CREATE (qc)-[n:QC_FOR{ t:  $t}]->(b) RETURN n").param("id", block_id)
            .param("t", t as u32)
            .param("qc_id", qc_id)).await.expect("Failed to create block").next().await.expect("Failed to create block");
    }

    pub async fn on_transaction_prepared_ready(
        &self,
        tx_id: u32,
        shard: Shard,
        in_qc: Arc<Qc>,
        t: u128,
    ) {
        self.client.execute(query("MATCH (q: Qc {id: $id}), (t: Transaction { id: $tx_id}) CREATE (t)-[n:PREPARED_READY_IN{ t:  $t}]->(q) RETURN n")
            .param("id", in_qc.id)
            .param("t", t as u32)
            .param("tx_id", tx_id)).await.expect("Failed to create block").next().await.expect("Failed to create block");
    }

    pub async fn on_transaction_prepared_waiting(
        &self,
        tx_id: u32,
        shard: Shard,
        in_qc: Arc<Qc>,
        t: u128,
    ) {
        self.client.execute(query("MATCH (q: Qc {id: $id}), (t: Transaction { id: $tx_id}) CREATE (t)-[n:PREPARED_WAITING_IN { t:  $t}]->(q) RETURN n")
            .param("id", in_qc.id)
            .param("t", t as u32)
            .param("tx_id", tx_id)).await.expect("Failed to create block").next().await.expect("Failed to create block");
    }

    pub async fn on_transaction_precommit_ready(
        &self,
        tx_id: u32,
        shard: Shard,
        in_qc: Arc<Qc>,
        t: u128,
    ) {
        self.client.execute(query("MATCH (q: Qc{id: $id}), (t: Transaction { id: $tx_id}) CREATE (t)-[n:PRECOMMIT_READY_IN { t:  $t}]->(q) RETURN n")
            .param("id", in_qc.id)
            .param("t", t as u32)
            .param("tx_id", tx_id)).await.expect("Failed to create block").next().await.expect("Failed to create block");
    }

    pub async fn on_transaction_precommit_waiting(
        &self,
        tx_id: u32,
        shard: Shard,
        in_qc: Arc<Qc>,
        t: u128,
    ) {
        self.client.execute(query("MATCH (q:Qc{id: $id}), (t: Transaction { id: $tx_id}) CREATE (t)-[n:PRECOMMIT_WAITING_IN { t:  $t}]->(q) RETURN n")
            .param("id", in_qc.id)
            .param("t", t as u32)
            .param("tx_id", tx_id)).await.expect("Failed to create block").next().await.expect("Failed to create block");
    }

    pub async fn on_transaction_committed(
        &self,
        tx_id: u32,
        shard: Shard,
        in_qc: Arc<Qc>,
        t: u128,
    ) {
        self.client.execute(query("MATCH (q: Qc {id: $id}), (t: Transaction { id: $tx_id}) CREATE (t)-[n:COMMITTED_IN { t:  $t}]->(q) RETURN n")
            .param("id", in_qc.id)
            .param("t", t as u32)
            .param("tx_id", tx_id)).await.expect("Failed to create block").next().await.expect("Failed to create block");
    }

    pub async fn on_message_sent(
        &self,
        from: u32,
        to: u32,
        message_id: u32,
        message: String,
        t: u128,
    ) {
        let mut res = self
            .client
            .execute(
                query(" CREATE (m:Message {id: $message_id, message:$message, t:$t})")
                    .param("to", format!("node_{}", to))
                    .param("t", t as u32)
                    .param("message_id", message_id)
                    .param("message", message),
            )
            .await
            .expect("Failed to create message");
        while let Ok(Some(row)) = res.next().await {
            // println!("message created");
        }

        let mut res = self.client.execute(
            query("MATCH (vn: Node { id: $from}), (m: Message {id: $message_id}) CREATE (vn)-[n:SENT { t:  $t}]->(m) RETURN n")
                .param("from", format!("node_{}", from))
                .param("to", format!("node_{}", to))
                .param("t", t as u32)
                .param("message_id", message_id)
        ).await.expect("Failed to create message");

        while let Ok(Some(row)) = res.next().await {
            // println!("message created");
        }
    }

    pub async fn on_request_block(&self, id: u32) {
        let mut lock = self.stats.write().await;
        lock.entry(id)
            .or_insert_with(|| Stats::default())
            .request_block += 1;
    }

    pub async fn on_leader_failure(&self, id: u32) {
        let mut lock = self.stats.write().await;
        lock.entry(id)
            .or_insert_with(|| Stats::default())
            .leader_failures += 1;
    }

    pub async fn print_stats(&self) {
        let lock = self.stats.read().await;
        for (id, stats) in lock.iter() {
            println!(
                "Stats for {}: leaves: {} block_requests: {} leader failures: {}",
                id, stats.leaves_created, stats.request_block, stats.leader_failures
            );
        }
    }
}

impl Debug for Subscriber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscriber")
    }
}

impl Subscriber {
    pub async fn connect() -> Self {
        // Create a Neo4j client
        let uri = "127.0.0.1:7687";
        let user = "neo4j";
        let pass = "password123";
        let client = Arc::new(Graph::new(&uri, user, pass).await.unwrap());

        // clear

        let mut res = client
            .execute(query("MATCH (n) DETACH DELETE n"))
            .await
            .expect("Failed to clear");
        while let Ok(Some(row)) = res.next().await {
            println!("cleared");
        }

        Self {
            client,
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
