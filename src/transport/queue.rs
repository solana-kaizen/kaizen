
// use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use ahash::HashMap;
use workflow_core::id::Id;
use workflow_allocator::prelude::*;
use workflow_allocator::transport::transaction::Transaction;
use workflow_allocator::transport::transaction::TransactionChain;
use workflow_allocator::transport::observer::Observer;
use workflow_allocator::result::Result;

#[derive(Clone)]
pub struct TransactionQueue {
    pub tx_chains : Arc<Mutex<Vec<Arc<TransactionChain>>>>,
    // pub map : BTreeMap<Pubkey, Arc<Mutex<Transaction>>>,

    pub observers : Arc<Mutex<HashMap<Id,Arc<dyn Observer>>>>
}

unsafe impl Send for TransactionQueue {}

impl TransactionQueue {
    pub fn new() -> TransactionQueue {
        TransactionQueue {
            tx_chains : Arc::new(Mutex::new(Vec::new())),
            observers : Arc::new(Mutex::new(HashMap::default()))
        }
    }

    pub fn register_observer(&self, id : &Id, observer : Arc<dyn Observer>) -> Result<()> {
        self.observers.lock()?.insert(id.clone(), observer);
        Ok(())
    }

    pub fn unregister_observer(&self, id : &Id) -> Result<()> {
        self.observers.lock()?.remove(id);
        Ok(())
    }


    // ~~~

    pub fn find_tx_chain(&self, transaction: &Arc<Transaction>) -> Result<Option<Arc<TransactionChain>>> {
        let tx_accounts = transaction.accounts()?;

        let pending = self.tx_chains.lock()?;
        for tx_chain in pending.iter() {
            let accounts = tx_chain.accounts()?;
            if accounts.intersection(&tx_accounts).count() > 0 {
                return Ok(Some(Arc::clone(tx_chain)));
            }
        }

        Ok(None)
    }

    pub async fn enqueue(&self, transaction : Arc<Transaction>) -> Result<()> {
    
        let tx_chain = {
            // check if tx chain exists and if it does, enqueue into it
            if let Some(tx_chain) = self.find_tx_chain(&transaction)? {
                tx_chain.enqueue(&transaction)?;
                self.observers.lock()?.values().for_each(|observer|{
                    observer.tx_created(&tx_chain.id, &transaction);
                });
                return Ok(())
            } 
            
            let tx_chain = Arc::new(TransactionChain::new());
            self.tx_chains.lock()?.push(tx_chain.clone());
            self.observers.lock()?.values().for_each(|observer|{
                observer.tx_chain_created(&tx_chain.id);
            });
        
            tx_chain
        };

        
        workflow_core::task::spawn(async move {
            
            
            // let tx_chain = 
            loop {
                let tx = tx_chain.dequeue_for_processing().unwrap();
                if let Some(tx) = tx {
                    
                    let transport = Transport::global().expect("Transport global is not available");
                    match transport.execute(&tx.instruction).await {
                        Ok(_) => { },
                        Err(err) => {
                        }
                    }

                } else {
                    break;
                }
                // let tx = tx_chain.inner.lock().unwrap().pending.first
            }

// Ok(())

        });

        // let result = transport.execute(&transaction.instruction).await;
        // match result {
        //     Ok(_) => {
        //         if let Some(observer) = &observer {
        //             observer.transaction_success(&tx_set.id, &transaction);
        //             if tx_set_created {
        //                 observer.transaction_set_complete(&tx_set.id);
        //             }
        //         }        
        //     },
        //     Err(err) => {
        //         if let Some(observer) = &observer {
        //             observer.transaction_failure(&tx_set.id, &transaction, &err);
        //         }
        //     }
        // }


        Ok(())
    }
}

