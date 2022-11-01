use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
use downcast::{downcast_sync,AnySync};
use workflow_allocator::result::Result;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub mod wasm;
        pub use wasm::*;
    } else if #[cfg(not(target_arch = "wasm32"))] {
        pub mod native;
        pub use native::*;
    }
}

#[derive(Debug)]
pub struct Adapter {
    pub name : String,
    pub icon : String,
    pub index : usize,
    pub detected: bool
}


#[async_trait(?Send)]
pub trait WalletInterface : AnySync {
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

downcast_sync!(dyn WalletInterface);
