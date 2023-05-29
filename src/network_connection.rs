use std::time::Duration;
use std::collections::VecDeque;
use crate::message::Message;

pub struct NetworkConnection{
    latency: Duration,
    messages: VecDeque<(u128, Message)>
}

impl NetworkConnection {
    fn new(latency: Duration) -> Self {
        Self {
            latency,
            messages: VecDeque::new()
        }
    }

    fn push_message(&mut self, message: Message, at_time: u128) {
        self.messages.push_back((at_time + self.latency.as_millis(), message));
    }

    fn peek(&self) -> Option<&(u128, Message)> {
        self.messages.front()
    }
}
