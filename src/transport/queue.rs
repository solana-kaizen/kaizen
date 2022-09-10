
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
use workflow_log::log_error;
use workflow_log::log_warning;

/// # TransactionQueue
/// 
/// TransactionQueue instance is able to receive multiple transactions
/// and register multiple Observer instances.
/// 
/// When receiving transaction, the Queue will create a "transaction chain"
/// for which it will start async processing task.  During this task
/// processing, other transactions can be submitted to the queue.
/// 
/// If the queue detects that transaction accounts are intersecting with
/// and existing chain, this transaction will be queued at the end of this
/// chain.  Otherwise, a new chain will be created.
/// 
/// Upon successful completion of all transactions in the chain, the chain
/// gets dropped and observers are notified via tx_chain_complete() notification.
/// 
/// If, however, transaction fails, the transaction will be re-added to the chain
/// as the first item and the chain will be left dangling. A dangling chain
/// can be discarded from the queue with `TransactionQueue::discard_chain(id:&Id)`
/// at which point `Observer::tx_chain_discarded(id:&Id)` will be called.
/// 
/// Please see workflow_allocator::transport::observer::Observer for details on how to
/// handle transaction chain and transaction notifications.
/// 



#[derive(Clone)]
pub struct TransactionQueue {
    pub tx_chains : Arc<Mutex<HashMap<Id,Arc<TransactionChain>>>>,
    pub observers : Arc<Mutex<HashMap<Id,Arc<dyn Observer>>>>
}

unsafe impl Send for TransactionQueue {}

impl TransactionQueue {
    pub fn new() -> TransactionQueue {
        TransactionQueue {
            tx_chains : Arc::new(Mutex::new(HashMap::default())),
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

    pub async fn discard_chain(&self, id : &Id) -> Result<()> {
        if let Some(tx_chain) = self.tx_chains.lock().unwrap().remove(&id) {
            for observer in self.observers()?.iter() {
                observer.tx_chain_discarded(&tx_chain).await;
            }
        }

        Ok(())
    }

    pub fn find_tx_chain_with_transaction(&self, tx: &Arc<Transaction>) -> Result<Option<Arc<TransactionChain>>> {
        let tx_id = tx.id;

        for (_,tx_chain) in self.tx_chains.lock()?.iter() {
            if tx_chain.inner.lock()?.pending.iter().position(|tx| tx.id == tx_id).is_some() {
                return Ok(Some(tx_chain.clone()));
            }
        }

        Ok(None)
    }

    pub fn find_tx_chain_account_intersection(&self, transaction: &Arc<Transaction>) -> Result<Option<Arc<TransactionChain>>> {
        let tx_accounts = transaction.accounts()?;

        let pending = self.tx_chains.lock()?;
        for (_,tx_chain) in pending.iter() {
            let accounts = tx_chain.accounts()?;
            if accounts.intersection(&tx_accounts).count() > 0 {
                return Ok(Some(tx_chain.clone()));
            }
        }

        Ok(None)
    }

    pub async fn enqueue(&self, transaction : Arc<Transaction>) -> Result<()> {
    
        let queue = self.clone();
        let tx_chain = {

            let tx_chain = match *transaction.status.lock()? {

                // should not occur - received already completed transaction
                TransactionStatus::Success => {
                    return Err(ErrorCode::TransactionAlreadyCompleted.into());
                },

                // retry mechanism - find existing chain, restart chain processing
                // if chain is not found, start new chain
                TransactionStatus::Timeout | TransactionStatus::Error(_) => {
                    if let Some(tx_chain) = queue.find_tx_chain_with_transaction(&transaction)? {
                        Some(tx_chain)
                    } else {
                        log_warning!("Unable to find transaction chain during transaction resubmission");
                        None
                    }
                },

                // new transaction, create a chain and excute chain processing
                TransactionStatus::Pending => {
                
                    // check if tx chain exists and if it does, enqueue into it
                    if let Some(tx_chain) = queue.find_tx_chain_account_intersection(&transaction)? {
                        tx_chain.enqueue(&transaction)?;
                        for observer in queue.observers()?.iter() {
                            observer.tx_created(&tx_chain, &transaction).await;
                        }
                        return Ok(())
                    }

                    None
                }
            };

            match tx_chain {
                // chain exists, chain processing will be restarted
                Some(tx_chain) => tx_chain,
                // chain does not exist, create new chain and start processing
                None => {
                    let tx_chain = Arc::new(TransactionChain::new());
                    queue.tx_chains.lock()?.insert(tx_chain.id.clone(), tx_chain.clone());
                    for observer in queue.observers()?.iter() {
                        observer.tx_chain_created(&tx_chain).await;
                    }
                
                    tx_chain
                }
            }
        };

        workflow_core::task::spawn(async move {
            match queue.process_transaction_chain_task(&tx_chain).await {
                Ok(_) => {

                    // on Success, transaction chain gets destroyed

                    if tx_chain.is_done().unwrap() {
                        for observer in queue.observers().unwrap().iter() {
                            observer.tx_chain_complete(&tx_chain).await;
                        }
                        
                        queue.tx_chains.lock().unwrap().remove(&tx_chain.id);
                    }
                }
                Err(err) => {

                    // on failure, transaction is re-inserted into the chain as first item
                    // and the chain is left dangling.  Failed transaction can be resubmitted
                    // resulting in restarting of the chain processing

                    log_error!("TransactionQueue::process_transaction_task failure: {}", err);
                }
            }

        });

        Ok(())
    }

    fn observers(&self) -> Result<Vec<Arc<dyn Observer>>> {
        let observers = self.observers.lock()?.values().cloned().collect();
        Ok(observers)
    }

    // main asynchronous transaction chain processing loop
    async fn process_transaction_chain_task(&self, tx_chain: &Arc<TransactionChain>) -> Result<()> {
        loop {
            let queue = self.clone();
            let observers = queue.observers()?;

            let tx = tx_chain.dequeue_for_processing().unwrap();
            if let Some(tx) = tx {
                
                let transport = Transport::global()?;
                match transport.execute(&tx.instruction).await {
                    Ok(_) => {
                        { *tx.status.lock().unwrap() = TransactionStatus::Success; }

                        tx_chain.set_as_complete(&tx).await?;
                        tx.sender.send(Ok(())).await?;

                        for observer in observers.iter() {
                            observer.tx_success(tx_chain, &tx).await;
                        }
                        
                        // at this point, transaction gets dropped...
                    },
                    Err(err) => {
                        { *tx.status.lock().unwrap() = TransactionStatus::Error(err.to_string()); }

                        // re-insert the trnansaction into the queue on first position
                        tx_chain.requeue_with_error(&tx, &err).await?;
                        tx.sender.send(Err(err.clone())).await?;

                        for observer in observers.iter() {
                            observer.tx_failure(tx_chain, &tx, err.clone()).await;
                        }

                        return Err(err);
                    }
                }
            } else {
                
                break;
            }
        }

        Ok(())
    }

}

