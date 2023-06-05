use crate::block::Block;
use crate::qc::Qc;
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
    NewView {
        id: u32,
        height: u32,
        high_qc: Arc<Qc>,
        from: u32,
    },
    Vote {
        id: u32,
        block_id: u32,
        block_height: u32,
        vote_by: u32,
    },
    RequestBlock {
        id: u32,
        block_id: u32,
        request_by: u32,
    },
    RequestBlockResponse {
        id: u32,
        block: Arc<Block>,
    },
}

impl Message {
    pub fn id(&self) -> u32 {
        match self {
            Message::Transaction { id, .. } => *id,
            Message::BlockProposal { id, .. } => *id,
            Message::Vote { id, .. } => *id,
            Message::RequestBlock { id, .. } => *id,
            Message::RequestBlockResponse { id, .. } => *id,
            Message::NewView { id, .. } => *id,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Transaction { id, tx } => write!(f, "Msg:Transaction: {}:{:?}", id, tx),
            Message::BlockProposal { id, block } => {
                write!(
                    f,
                    "Msg:BlockProposal: {} Block:{} shard: {} height:{} prep:{} precom:{}",
                    id,
                    block.id,
                    block.shard.0,
                    block.height,
                    block.prepare_txs.len(),
                    block.precommit_txs.len()
                )
            }
            Message::Vote { id, vote_by, .. } => write!(f, "Msg:Vote: {} from {}", id, vote_by),
            Message::RequestBlock { id, .. } => write!(f, "Msg:RequestBlock: {}", id),
            Message::RequestBlockResponse { id, .. } => {
                write!(f, "Msg:RequestBlockResponse: {}", id)
            }
            Message::NewView { id, from, .. } => {
                write!(f, "Msg:NewView: {} from: {}", id, from)
            }
        }
    }
}
