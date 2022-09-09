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
    name : String,
    pub signature : Option<Signature>,
    pub pubkey : Option<Pubkey>,
}

impl TransactionMeta {
    pub fn new_with_pubkey(name: &str, pubkey: &Pubkey) -> TransactionMeta {
        TransactionMeta {
            name: name.to_string(),
            signature: None,
            pubkey: Some(pubkey.clone())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn gather_pubkeys(&self) -> Result<HashSet<Pubkey>> {
        let mut pubkeys = HashSet::default();

        if let Some(pubkey) = self.meta.lock()?.pubkey.as_ref() {
            pubkeys.insert(pubkey.clone());
        }

        Ok(pubkeys)
    }

    pub async fn execute(&self) -> Result<()> {
        let transport = Transport::global()?;
        transport.execute(&self.instruction).await?;
        Ok(())
    }

    pub async fn execute_and_load<'this,T> (&self) -> Result<Option<ContainerReference<'this,T>>> 
    where T: workflow_allocator::container::Container<'this,'this> 
    {
        let pubkey = if let Some(pubkey) = self.meta.lock()?.pubkey.as_ref() {
            pubkey.clone()
        } else {
            panic!("Transaction::execute_and_load - missing pubkey");
        };

        let transport = Transport::global()?;
        transport.execute(&self.instruction).await?;
        load_container_with_transport::<T>(&transport,&pubkey).await
    }

}

pub struct TransactionSet {
    // name : String,
    transactions: Vec<Arc<Transaction>>,
}

impl TransactionSet {
    pub fn new(transactions : &[Arc<Transaction>]) -> TransactionSet {
        TransactionSet {
            transactions: transactions.to_vec()
        }
    }

    pub fn gather_pubkeys(&self) -> Result<HashSet<Pubkey>> {
        let mut pubkeys = HashSet::default();
        for transaction in self.transactions.iter() {
            let tx_pubkeys = transaction.gather_pubkeys()?;
            for pubkey in tx_pubkeys {
                pubkeys.insert(pubkey.clone());
            }
        }

        Ok(pubkeys)
    }
}
