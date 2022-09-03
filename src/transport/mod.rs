mod transaction;
mod queue;
mod observer;
mod lookup;

use std::sync::Arc;
// use async_std::sync::RwLock;
use async_trait::async_trait;
// use manual_future::ManualFuture;
use solana_program::pubkey::Pubkey;
// use solana_program::instruction::Instruction;
// use workflow_allocator::error::*;
use workflow_allocator::result::Result;
use workflow_allocator::accounts::AccountDataReference;
use solana_program::instruction::Instruction;
use downcast::{downcast_sync, AnySync};

#[async_trait(?Send)]
pub trait Interface : AnySync {
    fn get_authority_pubkey(&self) -> Result<Pubkey>;

    async fn execute(&self, instr : &Instruction) -> Result<()>;
    async fn lookup(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn lookup_local(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn lookup_remote(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;

    // async fn lookup_container<T>(&self, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;


    // async fn execute(self : &Arc<Self>, instr : &Instruction) -> Result<()>;
    // async fn lookup(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    // async fn lookup_local(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    // async fn lookup_remote(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;

}

downcast_sync!(dyn Interface);

mod config;
pub use config::*;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
