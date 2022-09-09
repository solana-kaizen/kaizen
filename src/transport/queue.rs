
// use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use workflow_allocator::prelude::*;
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

    pub fn find_set(&self, transaction: &Arc<Transaction>) -> Result<Option<Arc<TransactionSet>>> {
        let pubkeys = transaction.gather_pubkeys()?;

        let pending = self.pending.lock()?;
        for txset in pending.iter() {
            let txset_pubkeys = txset.gather_pubkeys()?;
            if txset_pubkeys.intersection(&pubkeys).count() > 0 {
                return Ok(Some(Arc::clone(txset)));
            }
        }

        Ok(None)
    }

    pub async fn enqueue(&self, transaction : Arc<Transaction>) -> Result<()> {
    
        // ^ TODO
        // let transactoin_set = Arc::new(TransactionSet::new(&[transaction]));
        // self.pending.lock()?.push(transactoin_set);

        let observer = self.observer.lock()?.as_ref().cloned();

        let (tx_set_created,tx_set) = if let Some(tx_set) = self.find_set(&transaction)? {
            (false, tx_set)
        } else {
            let tx_set = Arc::new(TransactionSet::new());
            (true, tx_set)
        };

        if let Some(observer) = &observer {
            if tx_set_created {
                observer.transaction_set_created(&tx_set.id);
            }
            observer.transaction_created(&tx_set.id, &transaction);
        }



        let transport = Transport::global()?;
        let result = transport.execute(&transaction.instruction).await;
        match result {
            Ok(_) => {
                if let Some(observer) = &observer {
                    observer.transaction_success(&tx_set.id, &transaction);
                    if tx_set_created {
                        observer.transaction_set_complete(&tx_set.id);
                    }
                }        
            },
            Err(err) => {
                if let Some(observer) = &observer {
                    observer.transaction_failure(&tx_set.id, &transaction, &err);
                }
            }
        }


        Ok(())
    }
}

