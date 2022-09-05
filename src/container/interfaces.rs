use solana_program::pubkey::Pubkey;
use async_trait::async_trait;
use workflow_allocator::result::Result;



#[async_trait(?Send)]
pub trait AccountAggregator {
    type Key;
    async fn locate_account_pubkeys(&self,key : &Self::Key) -> Result<Vec<Pubkey>>;
}
