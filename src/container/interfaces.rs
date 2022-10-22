// use async_trait::async_trait;
// use workflow_core::workflow_async_trait;
use workflow_allocator::prelude::*;
// use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use workflow_allocator::result::Result;



#[workflow_async_trait]
pub trait AsyncAccountAggregator {
    type Key;
    async fn writable_account_metas(&self,key : Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
    async fn readonly_account_metas(&self,key : Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
}

pub trait AccountAggregator {
    type Aggregator : AsyncAccountAggregator;
    fn aggregator(&self) -> Result<Arc<Self::Aggregator>>;
}

// #[async_trait(?Send)]
#[workflow_async_trait]
pub trait PdaCollectionCreator {
    async fn writable_account_meta(&self, program_id : &Pubkey) -> Result<(AccountMeta,u8)>;
    async fn writable_account_meta_range(&self, program_id : &Pubkey, items : usize) -> Result<Vec<(AccountMeta,u8)>>;
}

// #[async_trait(?Send)]
#[workflow_async_trait]

pub trait PdaCollectionAccessor {
    async fn writable_account_meta(&self, program_id : &Pubkey, index : usize) -> Result<AccountMeta>;
    async fn writable_account_meta_range(&self, program_id : &Pubkey, range : std::ops::Range<usize>) -> Result<Vec<AccountMeta>>;
    // async fn readonly_account_metas(&self, index: usize) -> Result<Vec<AccountMeta>>;
}
