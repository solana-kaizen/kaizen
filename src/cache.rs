use std::sync::Arc;
use async_std::sync::Mutex;
use solana_program::pubkey::Pubkey;
use crate::accounts::AccountDataReference;
use crate::result::Result;
// use crate::error::*;
use caches::{Cache as CacheTrait, RawLRU, DefaultEvictCallback,PutResult};
use ahash::RandomState;

type CacheNode = (AccountDataReference, usize);

const DEFAULT_CAPACITY : u64 = 1024u64 * 1024u64 * 512u64; // 512 megabytes

#[derive(Debug)]
pub struct CacheInner {
    capacity : u64,
    size : u64,
    items : usize,
    lru : RawLRU<Pubkey, CacheNode, DefaultEvictCallback, RandomState>,
}

#[derive(Clone)]
pub struct Cache(Arc<Mutex<CacheInner>>);

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.0.try_lock();
        match inner {
            Some(inner) => {
                write!(f, "Cache {{ size: {}, items: {}, capacity: {} }}", inner.size, inner.items, inner.capacity)?;
            },
            None => {
                write!(f, "Catche {{ <Unable to acquire lock> }}")?;
            }
        }
        Ok(())
    }
}

impl Cache {
    pub fn try_new_with_default_capacity() -> Result<Self> {
        Ok(Self::try_new_with_capacity(DEFAULT_CAPACITY)?)
    }

    pub fn try_new_with_capacity(capacity : u64) -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        let lru = RawLRU::with_hasher(1_000_000, RandomState::default())?;
        #[cfg(not(target_arch = "wasm32"))]
        // let lru = RawLRU::with_hasher(usize::MAX, RandomState::default())?;
        let lru = RawLRU::with_hasher(usize::MAX, RandomState::default())?;
        let inner = CacheInner {
            capacity,
            size : 0,
            items : 0,
            lru
        };
       Ok(Cache(Arc::new(Mutex::new(inner))))
    }

    pub async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<AccountDataReference>> {
        let mut cache = self.0.lock().await;
        let result = {
            // let mut lru = inner.lru.lock().await;
            match cache.lru.get(pubkey) {
                Some(node) => Some(node.0.clone()),
                None => None,
            }
        };
        Ok(result)
    }

    pub async fn store(&mut self, account_data : AccountDataReference) -> Result<()> {
        let (key, size) = {
            let inner = account_data.read().await;
            (inner.key.clone(), inner.space)
        };
        let mut cache = self.0.lock().await;
        cache.size += size as u64;
        cache.items += 1;
        let result = cache.lru.put(key, (account_data,size));
        match result {
            PutResult::Evicted { value, .. } => {
                let (_, size) = value;
                cache.size -= size as u64;
                cache.items -= 1;
            },
            _ => { }
        };

        while cache.size > cache.capacity {
            let node = cache.lru.remove_lru();
            match node {
                Some(node) => {
                    let (_,(_,size)) = node;
                    cache.size -= size as u64;
                    cache.items -= 1;
                },
                None => break,
            }
        }

        Ok(())
    }

}
