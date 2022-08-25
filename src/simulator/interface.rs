use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize,BorshDeserialize};
use std::sync::Arc;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
use solana_program::instruction;
use workflow_allocator::result::Result;
use workflow_allocator::accounts::AccountDataReference;
use workflow_allocator::error::ErrorCode;


#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ErrorData {
    pub message: String,
    pub code : ErrorCode,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub error : Option<ErrorData>,
    pub logs : Option<String>
}

impl ExecutionResponse {
    pub fn new(error: Option<ErrorData>, logs: Option<String>) -> Self {
        ExecutionResponse { error, logs }
    }
}

#[async_trait]
pub trait EmulatorInterface 
// where Arc<Self> : Sized// + Send + Sync + 'static
{
    // fn ctor(self : Arc<Self>) { }
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn execute(
        &self,
        instruction : &instruction::Instruction,
    ) -> Result<ExecutionResponse>;
    // async fn lookup(self : &Arc<Self>, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    // async fn execute(
    //     self : Arc<Self>,
    //     instruction : &instruction::Instruction,
    // ) -> Result<ExecutionResponse>;
}