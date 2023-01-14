// use async_trait::async_trait;
// use workflow_core::workflow_async_trait;
use kaizen::prelude::*;
// use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use kaizen::result::Result;

#[workflow_async_trait]
pub trait AsyncAccountAggregatorInterface {
    type Key;
    async fn writable_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
    async fn readonly_account_metas(&self, key: Option<&Self::Key>) -> Result<Vec<AccountMeta>>;
}

pub trait AccountAggregatorInterface {
    type Aggregator: AsyncAccountAggregatorInterface;
    fn aggregator(&self) -> Result<Arc<Self::Aggregator>>;
}

pub trait PdaCollectionCreatorInterface {
    type Creator: AsyncPdaCollectionCreatorInterface;
    fn creator(
        &self,
        _program_id: &Pubkey,
        _number_of_accounts: usize,
    ) -> Result<Arc<Self::Creator>>;
}

#[workflow_async_trait]
pub trait AsyncPdaCollectionCreatorInterface {
    async fn writable_accounts_meta(&self) -> Result<Vec<(AccountMeta, u8)>>;
}

pub trait PdaCollectionAccessorInterface {
    type Accessor: AsyncPdaCollectionAccessorInterface;
    fn accessor(
        &self,
        _program_id: &Pubkey,
        index_range: std::ops::Range<usize>,
    ) -> Result<Arc<Self::Accessor>>;
}

#[workflow_async_trait]
pub trait AsyncPdaCollectionAccessorInterface {
    async fn writable_accounts_meta(&self) -> Result<Vec<AccountMeta>>;
}
