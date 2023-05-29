use std::collections::HashMap;
use std::time::Duration;
use rand::Rng;
use crate::message::Message;
use crate::network_connection::NetworkConnection;
use crate::node_id::NodeId;

pub struct Network {
   connections: HashMap<NodeId, HashMap<NodeId, NetworkConnection>>
}

impl Network {
    fn new() -> Self {
        Self {
            connections: HashMap::new()
        }
    }

    fn add_connection(&mut self, from: NodeId, to: NodeId, min_latency: Duration, max_latency: Duration) {
        let latency  = if min_latency != max_latency {
             Duration::from_millis(rand::thread_rng().gen_range(min_latency.as_millis()..max_latency.as_millis()).try_into().unwrap())
        } else {
            min_latency
        };
        let min_id = std::cmp::min(from, to);
        let max_id = std::cmp::max(from, to);
            self.connections.entry(min_id).or_insert_with(HashMap::new).insert(max_id, NetworkConnection::new(latency));
    }

    fn send_message(&mut self, from: NodeId, to: NodeId, message: Message, current_time: u128) {
        let min_id = std::cmp::min(from, to);
        let max_id = std::cmp::max(from, to);
        let mut connection = self.connections.get_mut(&min_id).unwrap().get_mut(&max_id).unwrap();
        connection.push_message(message, current_time);
    }

    fn update(&mut self, current_time: u128) -> Vec<(NodeId, Message)> {
        let mut messages = Vec::new();
        for (from, connections) in &mut self.connections {
            for (to, connection) in connections {
                loop {
                    if let Some((time, message)) = connection.peek() {
                        if *time <= current_time {
                            let (_, message) = connection.messages.pop_front().unwrap();
                            messages.push((*to, message));
                        }  else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        messages
    }
}
