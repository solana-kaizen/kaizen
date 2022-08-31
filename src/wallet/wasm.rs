// use std::path::Path;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
// use solana_sdk::signature::{Keypair, read_keypair_file};
// use solana_sdk::signer::Signer;
use workflow_allocator::result::Result;


pub struct Wallet {
    
    
}


impl Wallet {

    pub fn try_new() -> Result<Wallet> {
        let wallet = Self {

        };

        Ok(wallet)
    }

}

#[async_trait]
impl super::Wallet for Wallet {

    fn is_connected(&self) -> bool {
        true
    }
    
    fn pubkey(&self) -> Result<Pubkey> {
        // Ok(self.keypair.pubkey())
        // temporary stub
        Ok(Pubkey::default())
    }

    async fn get_adapter_list(&self) -> Result<Option<Vec<super::Adapter>>> {
        Ok(None)
    }

    async fn connect(&self, _adapter: Option<super::Adapter>) -> Result<()> {
        Ok(())
    }

    // async fn get_balance(&self) -> Result<u64>;

}