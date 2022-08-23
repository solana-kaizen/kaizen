use cfg_if::cfg_if;
use std::sync::Arc;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
use workflow_allocator::accounts::AccountDataReference;
use workflow_allocator::result::Result;

mod memorystore;
pub use memorystore::MemoryStore;
cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod filestore;
        pub use filestore::FileStore;
    }
}

#[async_trait]
pub trait Store // : Sized
// where 
//     Self : Sized,
//     Arc<Self>: Sized
{
    async fn list(self : &Self) -> Result<()>;
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn store(&self, reference: &Arc<AccountDataReference>) -> Result<()>;
    async fn purge(&self, pubkey: &Pubkey) -> Result<()>;
}
