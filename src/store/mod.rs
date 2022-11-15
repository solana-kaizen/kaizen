use cfg_if::cfg_if;
use std::sync::Arc;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
use kaizen::accounts::{AccountDataReference,AccountDescriptorList};
use kaizen::result::Result;

mod memorystore;
pub use memorystore::MemoryStore;
cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod filestore;
        pub use filestore::FileStore;
    }
}

#[async_trait]
pub trait Store : Sync + Send
{
    async fn list(&self) -> Result<AccountDescriptorList>;
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn store(&self, reference: &Arc<AccountDataReference>) -> Result<()>;
    async fn purge(&self, pubkey: &Pubkey) -> Result<()>;
}
