use async_trait::async_trait;
use solana_program::instruction::AccountMeta;
use workflow_allocator::result::Result;



#[async_trait(?Send)]
pub trait AccountAggregator {
    type Key;
    async fn writable_account_metas(&self,key : Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
    async fn readonly_account_metas(&self,key : Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
}
