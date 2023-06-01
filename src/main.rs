use crate::block::Block;
use crate::committee_manager::CommitteeManager;
use crate::id_provider::IdProvider;
use crate::subscriber::Subscriber;
use crate::transaction::Shard;
use clap::Parser;
use cli::Cli;
use indexer::Indexer;
use message::Message;
use message_id_factory::MessageIdFactory;
use network::Network;
use network_connection::NetworkConnection;
use node_factory::NodeFactory;
use node_id::NodeId;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::process::id;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio;
use transaction::Transaction;
use transaction_generator::TransactionGenerator;
use validator_node::ValidatorNode;

mod block;
mod block_factory;
mod cli;
mod committee_manager;
mod id_provider;
mod indexer;
mod message;
mod message_id_factory;
mod network;
mod network_connection;
mod node_factory;
mod node_id;
mod qc;
mod subscriber;
mod transaction;
mod transaction_generator;
mod validator_node;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut vns = HashMap::new();
    let subscriber = Subscriber::connect().await;
    let mut network = Network::new(subscriber.clone());
    let id_provider = IdProvider::new();
    let indexer = Indexer::new(id_provider.next());
    subscriber.create_indexer(indexer.id).await;
    let cli = Arc::new(cli);
    let genesis = Arc::new(Block::genesis());
    let committee_manager = CommitteeManager::new();
    for s in 0..cli.num_shards {
        subscriber.create_shard(s).await;
    }
    for i in 0..cli.num_vns {
        let shard = Shard(i as u32 % cli.num_shards);
        let latency = Duration::from_millis(
            rand::thread_rng()
                .gen_range(cli.min_latency.as_millis() - 1..cli.max_latency.as_millis())
                .try_into()
                .unwrap(),
        );
        let vn = ValidatorNode::new(
            id_provider.next(),
            shard,
            cli.clone(),
            genesis.clone(),
            id_provider.clone(),
            committee_manager.clone(),
            subscriber.clone(),
            latency.as_millis(),
        );
        subscriber
            .create_vn(vn.id, vn.shard, latency.as_millis())
            .await;

        network.add_connection(indexer.id, vn.id, latency, latency);
        committee_manager.add_validator(vn.shard, vn.id).await;
        vns.insert(vn.id, vn);
    }

    for (_, vn) in &vns {
        for (_, vninner) in &vns {
            if vn.id == vninner.id {
                network.add_connection(
                    vn.id,
                    vninner.id,
                    Duration::from_millis(0),
                    Duration::from_millis(0),
                );
            } else {
                network.add_connection(
                    vn.id,
                    vninner.id,
                    Duration::from_millis((vn.base_latency + vninner.base_latency) as u64),
                    Duration::from_millis((vn.base_latency + vninner.base_latency) as u64) * 2,
                );
            }
        }
    }

    let mut transaction_generator = TransactionGenerator::new();
    let mut curr_time = 0;
    let time_step_millis = cli.time_per_step.as_millis();
    let num_steps = cli.num_steps as u128;

    loop {
        println!("Time: {:?}", curr_time);
        if let Some(transaction) = transaction_generator.next() {
            let transaction = Arc::new(transaction);
            println!("Transaction: {:?}", transaction);
            for (vn_id, _) in &vns {
                let m = Message::Transaction {
                    id: id_provider.next(),
                    tx: transaction.clone(),
                };

                println!("Sending transaction message: {} to vn: {:?}", m.id(), vn_id);
                network.send_message(indexer.id, *vn_id, m, curr_time).await;
            }
        }
        loop {
            let mut new_messages = false;
            // loop because there are loop backs
            let messages = network.update(curr_time);
            for (to, message) in messages {
                println!("Message: {} arrives at: {:?}", message, to);
                vns.get_mut(&to)
                    .expect("not found")
                    .deliver_message(message, curr_time);
            }
            for (_, vn) in &mut vns {
                let broadcasts = vn.update(curr_time).await;
                if !broadcasts.is_empty() {
                    new_messages = true;
                }
                for (to, message) in broadcasts {
                    network.send_message(vn.id, to, message, curr_time).await;
                }
            }
            if !new_messages {
                break;
            }
        }
        curr_time += time_step_millis;
        if curr_time / time_step_millis % (cli.print_stats_every as u128) == 0 {
            for (_, vn) in &vns {
                vn.print_stats();
            }
            subscriber.print_stats().await;
        }
        if curr_time > time_step_millis * num_steps {
            break;
        }
    }

    for (_, vn) in &vns {
        vn.print_stats();
    }

    subscriber.print_stats().await;
}
