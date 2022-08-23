use cfg_if::cfg_if;
use std::sync::Arc;
use solana_program::pubkey::Pubkey;
use crate::accounts::AccountDataReference;
use crate::result::Result;
use workflow_log::log_trace;

#[cfg(target_arch = "wasm32")]
use async_std::sync::Mutex;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use moka::unsync::Cache as MokaCache;
    } else {
        use moka::sync::Cache as MokaCache;
    }
}

const DEFAULT_CAPACITY : u64 = 1024u64 * 1024u64 * 256u64; // 256 megabytes

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub struct Cache {
            cache_store : Arc<Mutex<MokaCache<Pubkey,Arc<AccountDataReference>>>>
        }
    } else {
        pub struct Cache {
            cache_store : MokaCache<Pubkey,Arc<AccountDataReference>>
        }
    }
}
    
impl Cache {

    pub fn new_with_capacity(capacity: u64) -> Cache {
        log_trace!("init moka");
        let cache_store = MokaCache::builder()
        .weigher(|_key, reference: &Arc<AccountDataReference>| -> u32 {
            reference.data_len as u32
        })
        .max_capacity(capacity)
        .build();
        log_trace!("init moka ok");

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self { cache_store : Arc::new(Mutex::new(cache_store)) }
            } else {
                Self { cache_store }
            }
        }
    }

    pub fn new_with_default_capacity() -> Self {
        Self::new_with_capacity(DEFAULT_CAPACITY)
    }

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {

            #[inline(always)]
            pub async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
                Ok(self.cache_store.lock().await.get(pubkey).cloned())
            }
            
            #[inline(always)]
            pub async fn store(&mut self, reference : &Arc<AccountDataReference>) -> Result<()> {
                Ok(self.cache_store.lock().await.insert(*reference.key,reference.clone()))
            }

        } else {
            
            pub async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
                Ok(self.cache_store.get(pubkey))
            }
            
            #[inline(always)]
            pub async fn store(&mut self, reference : &Arc<AccountDataReference>) -> Result<()> {
                Ok(self.cache_store.insert(*reference.key,reference.clone()))
            }

        }
    }

}
