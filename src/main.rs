use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::{Duration, Instant};
use clap::Parser;
use rand::Rng;
use cli::Cli;
use indexer::Indexer;
use message::Message;
use message_id_factory::MessageIdFactory;
use network::Network;
use network_connection::NetworkConnection;
use node_factory::NodeFactory;
use node_id::NodeId;
use transaction::Transaction;
use transaction_generator::TransactionGenerator;
use validator_node::ValidatorNode;

mod cli;
mod validator_node;
mod transaction_generator;
mod transaction;
mod node_id;
mod node_factory;
mod network_connection;
mod network;
mod indexer;
mod message;
mod message_id_factory;


fn main() {
    let cli = Cli::parse();
    let mut vns = HashMap::new();
    let mut node_factory = NodeFactory::new();
    let mut network = Network::new();
    let mut message_factory = MessageIdFactory::new();
    let indexer = Indexer::new(node_factory.next());
    let cli = Arc::new(cli);
    for i in 0..cli.num_vns {
        let vn = ValidatorNode::new(node_factory.next(), 0, i, cli.num_vns, cli.clone());
        network.add_connection(indexer.id, vn.id, cli.min_latency.into(), cli.max_latency.into());
        vns.insert(vn.id, vn);
    }

    for (_, vn) in &vns {
        for (_, vninner) in &vns {
            if vn.id == vninner.id {
               network.add_connection(vn.id, vninner.id, Duration::from_millis(0), Duration::from_millis(0));
            }
            network.add_connection(vn.id, vninner.id, cli.min_latency.into(), cli.max_latency.into());
        }

    }

    let mut transaction_generator = TransactionGenerator::new();
    let mut curr_time = 0;
    let time_step_millis = 100;
    let num_steps = 4;
    loop {
        println!("Time: {:?}", curr_time);
        if let Some(transaction) = transaction_generator.next() {
            println!("Transaction: {:?}", transaction);
            for (vn_id, _) in &vns {
                let m =  Message::Transaction(message_factory.next(), transaction.clone());

                println!("Sending transaction message: {} to vn: {:?}", m.id(), vn_id);
                network.send_message(indexer.id, *vn_id, m, curr_time);
            }
        }
        let messages = network.update(curr_time);
        for (to, message) in messages {
            println!("Message: {} arrives at: {:?}", message, to);
            vns.get_mut(&to).expect("not found").deliver_message(message, curr_time);
        }
        for (_, vn) in &mut vns {
            vn.update(curr_time);
        }
        curr_time += time_step_millis;
        if curr_time > time_step_millis * num_steps {
            break;
        }
    }
}
