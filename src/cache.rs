//!
//! Application-level in-memory account data cache backed by [`Moka Cache`](moka::sync::Cache).
//! 

use crate::accounts::AccountDataReference;
use crate::result::Result;
use cfg_if::cfg_if;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use workflow_log::log_trace;

#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use moka::unsync::Cache as MokaCache;
    } else {
        use moka::sync::Cache as MokaCache;
    }
}

const DEFAULT_CAPACITY: u64 = 1024u64 * 1024u64 * 64u64; // 64 megabytes
                                                         // const DEFAULT_CAPACITY : u64 = 1024u64 * 1024u64 * 256u64; // 256 megabytes

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub struct Cache {
            cache_impl : Arc<Mutex<MokaCache<Pubkey,Arc<AccountDataReference>>>>
        }
    } else {
        pub struct Cache {
            cache_impl : MokaCache<Pubkey,Arc<AccountDataReference>>
        }
    }
}

impl Cache {
    pub fn new_with_capacity(capacity: u64) -> Cache {
        log_trace!(
            "init account data cache with {} MiB capacity",
            capacity / 1024 / 1024
        );
        let cache_impl = MokaCache::builder()
            .weigher(|_key, reference: &Arc<AccountDataReference>| -> u32 {
                reference.data_len as u32
            })
            .max_capacity(capacity)
            .build();

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self { cache_impl : Arc::new(Mutex::new(cache_impl)) }
            } else {
                Self { cache_impl }
            }
        }
    }

    pub fn new_with_default_capacity() -> Self {
        Self::new_with_capacity(DEFAULT_CAPACITY)
    }

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {

            #[inline(always)]
            pub fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
                Ok(self.cache_impl.lock()?.get(pubkey).cloned())
            }

            #[inline(always)]
            pub fn store(&self, reference : &Arc<AccountDataReference>) -> Result<()> {
                Ok(self.cache_impl.lock()?.insert(*reference.key,reference.clone()))
            }

            #[inline(always)]
            pub fn purge(&self, pubkey : Option<&Pubkey>) -> Result<()> {
                match pubkey {
                    Some(pubkey) => Ok(self.cache_impl.lock()?.invalidate(pubkey)),
                    None => Ok(self.cache_impl.lock()?.invalidate_all()),
                }
            }

        } else {

            #[inline(always)]
            pub fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>> {
                Ok(self.cache_impl.get(pubkey))
            }

            #[inline(always)]
            pub fn store(&self, reference : &Arc<AccountDataReference>) -> Result<()> {
                self.cache_impl.insert(*reference.key,reference.clone());
                Ok(())
            }

            #[inline(always)]
            pub fn purge(&self, pubkey: Option<&Pubkey>) -> Result<()> {
                match pubkey {
                    Some(pubkey) => {
                        self.cache_impl.invalidate(pubkey);
                    }
                    None => {
                        self.cache_impl.invalidate_all();
                    },
                }
                Ok(())
            }

        }
    }
}
