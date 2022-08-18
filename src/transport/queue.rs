
use std::collections::BTreeMap;
use std::sync::Arc;
use async_std::sync::RwLock;
use crate::prelude::*;
use crate::transport::transaction::WorkflowTransaction;

#[derive(Debug, Clone)]
pub struct TransactionQueue {
    pub pending : Vec<Arc<RwLock<WorkflowTransaction>>>,
    pub map : BTreeMap<Pubkey, Arc<RwLock<WorkflowTransaction>>>,
}

impl TransactionQueue {
    pub fn new() -> TransactionQueue {
        TransactionQueue {
            pending : Vec::new(),
            map : BTreeMap::new(),
        }
    }
}

