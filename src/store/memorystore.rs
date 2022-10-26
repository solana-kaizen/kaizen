use crate::accounts::AccountDescriptor;

use super::*;
// use async_std::fs;
// use std::env ;
// use async_std::path::Path;
// use async_std::path::PathBuf;
use std::sync::Arc;
use async_std::sync::RwLock; // {RwLock,RwLockReadGuard,RwLockWriteGuard};
use solana_program::pubkey::Pubkey;
// use borsh::*;
// use workflow_log::*;
// use crate::result::Result;
// use workflow_log::log_trace;
use workflow_allocator::error::*;
// use workflow_allocator::accounts::AccountData;
// use crate::accounts::AccountDataReference;
use ahash::AHashMap;
// use async_trait::async_trait;


// pub struct MemoryStoreInner {
//     pub map : AHashMap<Pubkey, Arc<AccountDataReference>>
// }

#[derive(Clone)]
pub struct MemoryStore {
    map : Arc<RwLock<AHashMap<Pubkey, Arc<AccountDataReference>>>>
}

static mut STORE : Option<MemoryStore> = None;

impl MemoryStore {

    // pub async fn read<'t>(&'t self) -> RwLockReadGuard<'t, MemoryStoreInner> {
    //     self.0.read().await
    // }

    // pub async fn write<'t>(&'t self) -> RwLockWriteGuard<'t, MemoryStoreInner> {
    //     self.0.write().await
    // }

    pub fn new_global() -> Result<MemoryStore> {
        let cache = unsafe { (&STORE).as_ref()};
        if cache.is_some() {
            return Err(error!("Store::new() already invoked"));
        }
        // let inner = MemoryStoreInner {
        //     map : AHashMap::default(),
        // };
        let store = MemoryStore {
            map : Arc::new(RwLock::new(AHashMap::default())),
        };
        // let store = Arc::new(Store {//Store::new_with_inner( StoreInner {
        //     map : Arc::new(RwLock::new(BTreeMap::new()))
        // });
        // let clone = cache.clone();
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

        // let store = Arc::new(Store {
        //     map : Arc::new(RwLock::new(BTreeMap::new()))
        // });
        Ok(store)
    }

    // pub async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<AccountDataReference>> {
    //     let reference = {
    //         match self.0.read().await.map.get(&pubkey) {
    //             Some(reference) => Some(reference.clone()),
    //             None => None,
    //         }
    //     };
    //     Ok(reference)
    // }

    // pub async fn store(&self, reference : AccountDataReference) -> Result<()> {
    //     self.0.write().await.map.insert(*reference.key, reference);
    //     Ok(())
    // }

    // pub fn try_store(&self, account_data : AccountData) -> Result<()> {
    //     let pubkey = account_data.key.clone();
    //     let reference = AccountDataReference::new(account_data);
    //     self.0.try_write()
    //         .ok_or(error!("Store::try_store() - unable to write lock store"))?
    //         .map.insert(pubkey, reference);
    //     Ok(())
    // }

    // pub async fn purge_if_exists(&self, pubkey : &Pubkey) -> Result<()> {
    //     self.0.write().await.map.remove(&pubkey);        
    //     Ok(())    
    // }


}

#[async_trait]
impl Store for MemoryStore {

    async fn list(&self) -> Result<AccountDescriptorList> {
        let map = self.map.read().await;
        let mut account_descriptors = Vec::new();
        // let mut seq: usize = 1;
        for (_pubkey, reference) in map.iter() {

            let account_data = reference.account_data.lock()?;
            let descriptor: AccountDescriptor = (&*account_data).into();
            account_descriptors.push(descriptor);

            // log_trace!("[store] [{:>8}] ... {}", seq, reference.account_data.lock()?.info());
            // seq += 1;
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
