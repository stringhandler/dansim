use crate::message::Message;
use crate::network_connection::NetworkConnection;
use crate::node_id::NodeId;
use crate::subscriber::Subscriber;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct Network {
    connections: HashMap<u32, HashMap<u32, NetworkConnection>>,
    subscriber: Subscriber,
}

impl Network {
    pub fn new(subscriber: Subscriber) -> Self {
        Self {
            connections: HashMap::new(),
            subscriber,
        }
    }

    pub fn add_connection(
        &mut self,
        from: u32,
        to: u32,
        min_latency: Duration,
        max_latency: Duration,
    ) {
        let latency = if min_latency != max_latency {
            Duration::from_millis(
                rand::thread_rng()
                    .gen_range(min_latency.as_millis()..max_latency.as_millis())
                    .try_into()
                    .unwrap(),
            )
        } else {
            min_latency
        };

        self.connections
            .entry(from)
            .or_insert_with(HashMap::new)
            .insert(to, NetworkConnection::new(latency));
        self.connections
            .entry(to)
            .or_insert_with(HashMap::new)
            .insert(from, NetworkConnection::new(latency));
    }

    pub async fn send_message(&mut self, from: u32, to: u32, message: Message, current_time: u128) {
        let mut connection = self
            .connections
            .get_mut(&from)
            .unwrap()
            .get_mut(&to)
            .unwrap();
        self.subscriber
            .on_message_sent(from, to, message.id(), message.to_string(), current_time)
            .await;
        println!("{} Sent -> {}: {}", from, to, message.to_string());
        connection.push_message(message, current_time);
    }

    pub fn update(&mut self, current_time: u128) -> Vec<(u32, Message)> {
        let mut messages = Vec::new();
        for (from, connections) in &mut self.connections {
            for (to, connection) in connections {
                loop {
                    if let Some((time, message)) = connection.peek() {
                        if *time <= current_time {
                            let (_, message) = connection.messages.pop_front().unwrap();
                            messages.push((*to, message));
                        } else {
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
