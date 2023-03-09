//!
//! Platform-neutral Solana network and authority interface (Solana SDK on native and Web3 APIs in WASM-browser)
//!

mod lookup;
use downcast::{downcast_sync, AnySync};
use kaizen::accounts::AccountDataReference;
use kaizen::result::Result;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use workflow_core::id::Id;
use workflow_core::workflow_async_trait;

#[workflow_async_trait]
pub trait Interface: AnySync {
    fn get_authority_pubkey(&self) -> Result<Pubkey>;

    async fn execute(&self, instr: &Instruction) -> Result<()>;
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn lookup_local(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn lookup_remote(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn post(&self, tx: Arc<Transaction>) -> Result<()>;
    async fn discard_chain(&self, id: &Id) -> Result<()>;
    async fn post_multiple(&self, tx: Vec<Arc<Transaction>>) -> Result<()>;

    fn purge(&self, pubkey: Option<&Pubkey>) -> Result<()>;
}

downcast_sync!(dyn Interface);

mod config;
pub use config::*;

mod loaders;
pub use loaders::*;

mod reflector;
pub use reflector::*;

mod transaction;
pub use transaction::*;

mod queue;
pub use queue::*;

mod observer;
pub use observer::*;
pub mod api;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
