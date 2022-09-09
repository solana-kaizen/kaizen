
// use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
// use crate::prelude::*;
use workflow_allocator::transport::transaction::Transaction;
use workflow_allocator::transport::transaction::TransactionSet;
use workflow_allocator::transport::observer::Observer;
use workflow_allocator::result::Result;

#[derive(Clone)]
pub struct TransactionQueue {
    pub pending : Arc<Mutex<Vec<Arc<TransactionSet>>>>,
    // pub map : BTreeMap<Pubkey, Arc<Mutex<Transaction>>>,

    pub observer : Arc<Mutex<Option<Arc<dyn Observer>>>>
}

impl TransactionQueue {
    pub fn new() -> TransactionQueue {
        TransactionQueue {
            pending : Arc::new(Mutex::new(Vec::new())),
            observer : Arc::new(Mutex::new(None))
            // map : BTreeMap::new(),
        }
    }

    pub fn observer(&self) -> Result<Option<Arc<dyn Observer>>> {
        Ok(self.observer.lock()?.as_ref().cloned())
    }

    pub fn register_observer(&self, observer : Option<Arc<dyn Observer>>) -> Result<()> {
        *self.observer.lock()? = observer;
        Ok(())
    }

    pub fn unregister_observer(&self) -> Result<()> {
        *self.observer.lock()? = None;
        Ok(())
    }

    // ~~~

    pub async fn enqueue(&self, transaction : Arc<Transaction>) -> Result<()> {
    
        let transactoin_set = Arc::new(TransactionSet::new(&[transaction]));

        self.pending.lock()?.push(transactoin_set);



        Ok(())
    }
}

