use crate::message::Message;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug)]
pub struct NetworkConnection {
    pub latency: Duration,
    pub messages: VecDeque<(u128, Message)>,
}

impl NetworkConnection {
    pub fn new(latency: Duration) -> Self {
        Self {
            latency,
            messages: VecDeque::new(),
        }
    }

    pub fn push_message(&mut self, message: Message, at_time: u128) {
        self.messages
            .push_back((at_time + self.latency.as_millis(), message));
    }

    pub fn peek(&self) -> Option<&(u128, Message)> {
        self.messages.front()
    }
}
