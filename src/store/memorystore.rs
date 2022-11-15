use crate::accounts::AccountDescriptor;

use super::*;
use std::sync::Arc;
use async_std::sync::RwLock;
use solana_program::pubkey::Pubkey;
use kaizen::error::*;
use ahash::AHashMap;

#[derive(Clone)]
pub struct MemoryStore {
    map : Arc<RwLock<AHashMap<Pubkey, Arc<AccountDataReference>>>>
}

static mut STORE : Option<MemoryStore> = None;

impl MemoryStore {

    pub fn new_global() -> Result<MemoryStore> {
        let cache = unsafe { (&STORE).as_ref()};
        if cache.is_some() {
            return Err(error!("Store::new() already invoked"));
        }
        let store = MemoryStore {
            map : Arc::new(RwLock::new(AHashMap::default())),
        };
        unsafe { STORE = Some(store.clone()); }
        Ok(store)
    }

    pub fn global() -> Result<MemoryStore> {

        let cache = unsafe { (&STORE).as_ref()};
        match cache {
            Some(cache) => Ok(cache.clone()),
            None => {
                Ok(MemoryStore::new_global()?)
            }
        }
    }

    pub fn new_local() -> Result<MemoryStore> {
        let store = MemoryStore {
            map : Arc::new(RwLock::new(AHashMap::default())),
        };

        Ok(store)
    }

}

#[async_trait]
impl Store for MemoryStore {

    async fn list(&self) -> Result<AccountDescriptorList> {
        let map = self.map.read().await;
        let mut account_descriptors = Vec::new();
        for (_pubkey, reference) in map.iter() {

            let account_data = reference.account_data.lock()?;
            let descriptor: AccountDescriptor = (&*account_data).into();
            account_descriptors.push(descriptor);
        }
        Ok(AccountDescriptorList::new(account_descriptors)) 
    }

    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
        Ok(self.map.read().await.get(&pubkey).cloned())
    }
    async fn store(&self, reference : &Arc<AccountDataReference>) -> Result<()> {
        self.map.write().await.insert(*reference.key, reference.clone());
        Ok(())
    }
    async fn purge(&self, pubkey : &Pubkey) -> Result<()> {
        self.map.write().await.remove(&pubkey);        
        Ok(())
    }
}
