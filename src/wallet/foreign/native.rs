// use std::sync::Arc;
use std::path::Path;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
// use solana_program::instruction::Instruction;
use solana_sdk::signature::{Keypair, read_keypair_file};
use solana_sdk::signer::Signer;
// use solana_sdk::transaction::Transaction;
// use downcast::{downcast_sync,AnySync};
use workflow_allocator::result::Result;


// #[derive(Clone)]
pub struct Wallet {
    // keypair : Arc<Keypair>,
    keypair : Keypair,
}


impl Wallet {

    pub fn try_new() -> Result<Wallet> {
        let home = home::home_dir()
            .expect("Wallet: unable to get home directory");
        let home = Path::new(&home);
        let payer_kp_path = home.join(".config/solana/id.json");
    
        let keypair = read_keypair_file(payer_kp_path)
            .expect("Couldn't read authority keypair from '~/.config/solana/id.json'");
        
        let wallet = Self {
            // keypair : Arc::new(keypair),
            keypair //: keypair),
        };

        Ok(wallet)
    }

    // pub fn keypair<'wallet>(&'wallet self) -> &'wallet Arc<Keypair> {
    pub fn keypair<'wallet>(&'wallet self) -> &'wallet Keypair {
        &self.keypair
    }
}

#[async_trait(?Send)]
impl super::WalletInterface for Wallet {

    fn is_connected(&self) -> bool {
        true
    }
    
    fn pubkey(&self) -> Result<Pubkey> {
        Ok(self.keypair.pubkey())
    }

    async fn get_adapter_list(&self) -> Result<Option<Vec<super::Adapter>>> {
        Ok(None)
    }

    async fn connect(&self, _adapter: Option<super::Adapter>) -> Result<()> {
        Ok(())
    }

    // async fn get_balance(&self) -> Result<u64>;

}