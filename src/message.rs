use std::fmt::{Display, Formatter};
use crate::transaction::Transaction;

pub enum Message {
    Transaction(usize, Transaction),
}

impl Message {
    fn id(&self) -> usize {
        match self {
            Message::Transaction(id, _) => *id
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Transaction(id, transaction) => write!(f, "Msg:Transaction: {}:{:?}", id, transaction)
        }
    }
}
