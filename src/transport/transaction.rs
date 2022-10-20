use std::sync::Mutex;
use ahash::HashSet;
use serde::{ Serialize, Deserialize };
use solana_sdk::signature::Signature;
use workflow_core::id::Id;
use workflow_core::channel::*;
use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;
use workflow_allocator::error::Error;

pub type TransactionResult = Result<()>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Success,
    Timeout,
    Error(String)
}

impl ToString for TransactionStatus{
    fn to_string(&self) -> String {
        match self{
            TransactionStatus::Pending=>"Pending".to_string(),
            TransactionStatus::Success=>"Success".to_string(),
            TransactionStatus::Timeout=>"Timeout".to_string(),
            TransactionStatus::Error(e)=>e.clone()
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMeta {
    /// Transaction caption
    name : String,
    /// Optional transaction signature during processing
    pub signature : Option<Signature>,
    /// Accounts affected by this transaction
    pub accounts : Vec<Pubkey>,
}

impl TransactionMeta {
    pub fn new_without_accounts(name: &str) -> TransactionMeta {
        TransactionMeta {
            name: name.to_string(),
            signature: None,
            accounts : Vec::new(),
        }
    }

    pub fn new_with_accounts(name: &str, accounts: &[&Pubkey]) -> TransactionMeta {
        TransactionMeta {
            name: name.to_string(),
            signature: None,
            accounts : accounts.iter().map(|pk|*pk.clone()).collect::<Vec<Pubkey>>(),
        }
    }
}

#[derive(Debug, Clone)] //, Serialize, Deserialize)]
pub struct Transaction {
    pub id : Id,
    pub instruction : Instruction,
    pub status : Arc<Mutex<TransactionStatus>>,
    pub meta : Arc<Mutex<TransactionMeta>>,
    pub receiver : Receiver<TransactionResult>,
    pub sender : Sender<TransactionResult>,
}

impl Transaction {

    pub fn new_without_accounts(name: &str, instruction: Instruction) -> Transaction {
        let meta = TransactionMeta::new_without_accounts(name);
        let (sender,receiver) = unbounded::<TransactionResult>();
        Transaction {
            id : Id::new(),
            status : Arc::new(Mutex::new(TransactionStatus::Pending)),
            meta : Arc::new(Mutex::new(meta)),
            instruction,
            sender,
            receiver,
        }
    }
    
    pub fn new_with_accounts(name: &str, accounts: &[&Pubkey], instruction: Instruction) -> Transaction {

        let meta = TransactionMeta::new_with_accounts(name, accounts);

        let (sender,receiver) = unbounded::<TransactionResult>();

        Transaction {
            id : Id::new(),
            status : Arc::new(Mutex::new(TransactionStatus::Pending)),
            meta : Arc::new(Mutex::new(meta)),
            instruction,
            sender,
            receiver,
        }
    }

    pub fn accounts(&self) -> Result<HashSet<Pubkey>> {
        let mut accounts = HashSet::default();
        let meta = self.meta.lock()?;

        for pubkey in meta.accounts.iter() {
            accounts.insert(pubkey.clone());
        }

        Ok(accounts)
    }

    pub async fn execute(&self) -> Result<()> {
        let transport = Transport::global()?;
        transport.execute(&self.instruction).await?;
        Ok(())
    }

    /// For *Create* operations it is assumed that the 
    /// resulting account is always at position [0]
    pub fn target_account(&self) -> Result<Pubkey> {
        let meta = self.meta.lock()?;
        if meta.accounts.is_empty() {
            panic!("Transaction::target_account(): missing target account");
        } else {
            Ok(meta.accounts[0].clone())
        }
    }

    /// Used for unit tests
    pub async fn execute_and_load<'this,T> (&self) -> Result<Option<ContainerReference<'this,T>>> 
    where T: workflow_allocator::container::Container<'this,'this> 
    {
        let pubkey = self.target_account()?;
        let transport = Transport::global()?;
        transport.execute(&self.instruction).await?;
        load_container_with_transport::<T>(&transport,&pubkey).await
    }

}

pub struct TransactionChainInner {
    pub pending: Vec<Arc<Transaction>>,
    pub complete: Vec<Arc<Transaction>>,
    pub accounts: HashSet<Pubkey>
}

impl TransactionChainInner {
    pub fn new() -> TransactionChainInner {
        TransactionChainInner {
            pending: Vec::new(),
            complete: Vec::new(),
            accounts: HashSet::default()
        }
    }
}

pub struct TransactionChain {
    pub id : Id,
    pub inner : Arc<Mutex<TransactionChainInner>>,
}

impl TransactionChain {
    pub fn new() -> TransactionChain {
        TransactionChain {
            id : Id::new(),
            inner : Arc::new(Mutex::new(TransactionChainInner::new())),
        }
    }
    pub fn extend_with(&mut self, transactions : &[Arc<Transaction>]) -> Result<()> {
        let mut inner = self.inner.lock()?;
        for transaction in transactions.iter() {
            inner.accounts.extend(&transaction.accounts()?);
        }
        inner.pending.extend_from_slice(transactions);
        Ok(())
    }

    pub fn accounts(&self) -> Result<HashSet<Pubkey>> {
        Ok(self.inner.lock()?.accounts.clone())
    }

    pub fn is_done(&self) -> Result<bool> {
        Ok(self.inner.lock()?.pending.is_empty())
    }

    pub fn enqueue(&self, transaction : &Arc<Transaction>) -> Result<()> {
        let mut inner = self.inner.lock()?;
        inner.pending.push(transaction.clone());
        Ok(())
    }

    pub fn dequeue_for_processing(&self) -> Result<Option<Arc<Transaction>>> {
        let mut inner = self.inner.lock()?;
        if inner.pending.is_empty() {
            Ok(None)
        } else {
            Ok(Some(inner.pending.remove(0)))
        }
    }
    
    pub async fn requeue_with_error(&self, transaction : &Arc<Transaction>, _err : &Error) -> Result<()> {
        let mut inner = self.inner.lock()?;
        inner.pending.insert(0, transaction.clone());
        Ok(())
    }

    pub async fn set_as_complete(&self, transaction : &Arc<Transaction>) -> Result<()> {
        let mut inner = self.inner.lock()?;
        inner.complete.push(transaction.clone());
        Ok(())
    }

}
