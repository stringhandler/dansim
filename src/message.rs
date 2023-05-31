use crate::block::Block;
use crate::transaction::Transaction;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug)]
pub enum Message {
    Transaction {
        id: u32,
        tx: Arc<Transaction>,
    },
    BlockProposal {
        id: u32,
        block: Arc<Block>,
    },
    Vote {
        id: u32,
        block_id: u32,
        block_height: u32,
        vote_by: u32,
    },
}

impl Message {
    pub fn id(&self) -> u32 {
        match self {
            Message::Transaction { id, .. } => *id,
            Message::BlockProposal { id, .. } => *id,
            Message::Vote { id, .. } => *id,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Transaction { id, tx } => write!(f, "Msg:Transaction: {}:{:?}", id, tx),
            Message::BlockProposal { id, block } => {
                write!(f, "Msg:BlockProposal: {} Block:{}", id, block.id)
            }
            Message::Vote { id, .. } => write!(f, "Msg:Vote: {}", id),
        }
    }
}
