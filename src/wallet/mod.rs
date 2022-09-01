use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
// use solana_program::instruction::Instruction;
// use solana_sdk::transaction::Transaction;
use downcast::{downcast_sync,AnySync};
use workflow_allocator::result::Result;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[derive(Debug)]
pub struct Adapter {
    pub name : String,
    pub icon : String,
    pub index : usize,
    pub detected: bool
}


#[async_trait]
pub trait Wallet : AnySync {
    fn is_connected(&self) -> bool;
    fn pubkey(&self) -> Result<Pubkey>;
    async fn get_adapter_list(&self) -> Result<Option<Vec<Adapter>>>;
    async fn connect(&self, adapter: Option<Adapter>) -> Result<()>;
    // belongs to transport... async fn get_balance(&self) -> Result<u64>;

    // ^ TODO - to sign, we should downcast_arc to native struct...

    // #[cfg(not(target_arch = "wasm32"))]
    // async fn sign(&self, instruction: &Instruction) -> Result<Transaction>;
    
    // #[cfg(target_arch = "wasm32")]
    // async fn sign(&self, instruction: &Instruction) -> Result<JsValue>;

}

downcast_sync!(dyn Wallet);
