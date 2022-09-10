use std::sync::Mutex;
use ahash::HashSet;
use serde::{ Serialize, Deserialize };
use solana_sdk::signature::Signature;
use workflow_core::id::Id;
use workflow_allocator::prelude::*;
use workflow_allocator::result::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Success,
    Timeout,
    Failure
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
    pub fn new_with_pubkey(name: &str, pubkey: &Pubkey) -> TransactionMeta {
        TransactionMeta {
            name: name.to_string(),
            signature: None,
            // pubkey: Some(pubkey.clone()),
            accounts : vec![pubkey.clone()],
        }
    }
}

#[derive(Debug, Clone)] //, Serialize, Deserialize)]
pub struct Transaction {
    pub id : Id,
    pub instruction : Instruction,
    pub status : Arc<Mutex<TransactionStatus>>,
    pub meta : Arc<Mutex<TransactionMeta>>,

    // ^ targets Vec<Pubkey> ???
    // ^ targets Vec<Pubkey> ???
    // ^ targets Vec<Pubkey> ???
}

impl Transaction {
    pub fn new_with_pubkey(name: &str, pubkey: &Pubkey, instruction: Instruction) -> Transaction {

        let meta = TransactionMeta::new_with_pubkey(name, pubkey);

        Transaction {
            id : Id::new(),
            status : Arc::new(Mutex::new(TransactionStatus::Pending)),
            meta : Arc::new(Mutex::new(meta)),
            instruction,
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

    pub fn set_as_complete(&self, transaction : &Arc<Transaction>) -> Result<()> {
        let mut inner = self.inner.lock()?;
        inner.complete.push(transaction.clone());
        Ok(())
    }

}
