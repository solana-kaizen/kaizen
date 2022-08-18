use crate::{prelude::*, generate_random_pubkey};
use serde::{ Serialize, Deserialize };
use solana_sdk::signature::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Success,
    Timeout,
    Failure
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMeta {
    descr : String,
    // TODO: create timestamp?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransaction {
    pub id : Pubkey,
    pub status : TransactionStatus,
    // pub status : RefCell<TransactionStatus>,
    pub instruction : Instruction,
    pub signature : Option<Signature>,
    pub meta : TransactionMeta,
}

impl WorkflowTransaction {
    pub fn new(instruction: Instruction, meta: TransactionMeta) -> WorkflowTransaction {
        let signature : Option<Signature> = None;
        WorkflowTransaction {
            id : generate_random_pubkey(),
            instruction,
            meta,
            status : TransactionStatus::Pending,
            signature,
        }
    }
}