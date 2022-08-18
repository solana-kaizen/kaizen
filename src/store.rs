use std::sync::Arc;
use async_std::sync::{RwLock,RwLockReadGuard,RwLockWriteGuard};
use solana_program::pubkey::Pubkey;
use workflow_log::*;
use crate::result::Result;
use crate::error::*;
use crate::accounts::AccountData;
use crate::accounts::AccountDataReference;
use ahash::AHashMap;

pub struct StoreInner {
    pub map : AHashMap<Pubkey, AccountDataReference>
}

#[derive(Clone)]
pub struct Store(Arc<RwLock<StoreInner>>);

static mut STORE : Option<Store> = None;

impl Store {

    pub async fn read<'t>(&'t self) -> RwLockReadGuard<'t, StoreInner> {
        self.0.read().await
    }

    pub async fn write<'t>(&'t self) -> RwLockWriteGuard<'t, StoreInner> {
        self.0.write().await
    }

    pub fn new_global() -> Result<Store> {
        let cache = unsafe { (&STORE).as_ref()};
        if cache.is_some() {
            return Err(error!("Store::new() already invoked"));
        }
        let inner = StoreInner {
            map : AHashMap::default(),
        };
        let store = Store(Arc::new(RwLock::new(inner)));
        // let store = Arc::new(Store {//Store::new_with_inner( StoreInner {
        //     map : Arc::new(RwLock::new(BTreeMap::new()))
        // });
        // let clone = cache.clone();
        unsafe { STORE = Some(store.clone()); }
        Ok(store)
    }

    pub fn global() -> Result<Store> {

        let cache = unsafe { (&STORE).as_ref()};
        match cache {
            Some(cache) => Ok(cache.clone()),
            None => {
                Ok(Store::new_global()?)
            }
        }
    }

    pub fn new_local() -> Result<Store> {
        let inner = StoreInner {
            map : AHashMap::default(),
        };
        let store = Store(Arc::new(RwLock::new(inner)));

        // let store = Arc::new(Store {
        //     map : Arc::new(RwLock::new(BTreeMap::new()))
        // });
        Ok(store)
    }

    /// Lookup account data in the local store
    pub async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<AccountDataReference>> {
        let account_data = {
            match self.0.read().await.map.get(&pubkey) {
                Some(account_data) => Some(account_data.clone()),
                None => None,
            }
        };
        Ok(account_data)
    }

    /// Store account data into the local cache
    pub async fn store(&self, account_data : AccountDataReference) -> Result<()> {
        let pubkey = account_data.read().await.key.clone();
        self.0.write().await.map.insert(pubkey, account_data);
        Ok(())
    }

    pub fn try_store(&self, account_data : AccountData) -> Result<()> {
        let pubkey = account_data.key.clone();
        let account_data = Arc::new(RwLock::new(account_data));
        self.0.try_write()
            .ok_or(error!("Store::try_store() - unable to write lock store"))?
            .map.insert(pubkey, account_data);
        Ok(())
    }

    pub async fn purge_if_exists(&self, pubkey : &Pubkey) -> Result<()> {
        self.0.write().await.map.remove(&pubkey);        
        Ok(())    
    }

    pub async fn list(&self) -> Result<()> {
        let inner = self.0.read().await;
        let mut seq: usize = 1;
        for (_pubkey, account_data) in inner.map.iter() {
            log_trace!("[store] [{:>8}] ... {}", seq, account_data.read().await.info()?);
            seq += 1;
        }
        Ok(())
    }

}
