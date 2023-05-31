use crate::block::Block;
use crate::transaction::Shard;
use neo4rs::query;
use neo4rs::Graph;
use neo4rs::Node;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Clone)]
pub struct Subscriber {
    client: Arc<Graph>,
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
            println!("leaf created");
        }

        let mut res = self.client.execute(query("MATCH (b: Block {id: $id}), (v: VN { id: $vn_id}) CREATE (v)-[n:PROPOSES { t:  $time}]->(b) RETURN n")
            .param("id", block.id)
            .param("time", time as u32)
            .param("vn_id", format!("vn_{}", block.proposed_by))).await.expect("Failed to create block");
        while let Ok(Some(row)) = res.next().await {
            println!("rel created");
        }
    }

    pub async fn on_vote(&self, vn_id: u32, block_id: u32, t: u128) {
        let mut res = self.client.execute(query("MATCH (b: Block {id: $id}), (v: VN { id: $vn_id}) CREATE (v)-[n:VOTES { t:  $t}]->(b) RETURN n")
            .param("id", block_id)
        .param("t", t as u32)
            .param("vn_id", format!("vn_{}", vn_id))).await.expect("Failed to create block");

        while let Ok(Some(row)) = res.next().await {
            println!("vote created");
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
            println!("shard created");
        }
    }

    pub async fn create_vn(&self, vn_id: u32, shard_id: Shard) {
        let mut res = self
            .client
            .execute(query("CREATE (vn: VN { id: $id })").param("id", format!("vn_{}", vn_id)))
            .await
            .expect("Failed to create vn");
        while let Ok(Some(row)) = res.next().await {
            println!("vn created");
        }
        dbg!(shard_id.0);
        let mut res = self.client
            .execute(
                query("MATCH (vn: VN { id: $vn_id}), (s:Shard { id: $id })  CREATE (vn)-[n:BELONGS_TO]->(s) RETURN n")
                    .param("id", format!("shard_{}", shard_id.0))
                    .param("vn_id", format!("vn_{}", vn_id)),
            )
            .await
            .expect("Could not link to shard");

        while let Ok(Some(row)) = res.next().await {
            println!("rel created");
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

        Self { client }
    }
}
