use async_trait::async_trait;
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use workflow_allocator::result::Result;



#[async_trait(?Send)]
pub trait AccountAggregator {
    type Key;
    async fn writable_account_metas(&self,key : Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
    async fn readonly_account_metas(&self,key : Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
}


#[async_trait(?Send)]
pub trait PdaCollectionBuilder {
    // type Key;
    async fn writable_account_meta(&self, program_id : &Pubkey) -> Result<(AccountMeta,u8)>;
    async fn writable_account_meta_range(&self, program_id : &Pubkey, items : usize) -> Result<Vec<(AccountMeta,u8)>>;
    // async fn readonly_account_metas(&self, index: usize) -> Result<Vec<AccountMeta>>;
}
