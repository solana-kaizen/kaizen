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

#[async_trait(?Send)]
pub trait Interface {
    fn get_authority_pubkey(&self) -> Result<Pubkey>;
    // async fn execute(&self, instr : &Arc<Instruction>) -> Result<()>;
    async fn execute(self : &Arc<Self>, instr : &Instruction) -> Result<()>;
    async fn lookup(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn lookup_local(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    // async fn lookup_remote(self : Arc<Self>, pubkey:&Pubkey) -> ManualFuture<Result<Option<Arc<RwLock<AccountData>>>>>;

    async fn lookup_remote(self : &Arc<Self>, pubkey:&Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
}

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
