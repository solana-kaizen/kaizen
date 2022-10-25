use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize,BorshDeserialize};
use std::sync::Arc;
use async_trait::async_trait;
use solana_program::pubkey::Pubkey;
use solana_program::instruction;
use workflow_allocator::result::Result;
use workflow_allocator::accounts::{AccountDataReference,AccountDescriptorList};
use downcast::{downcast_sync, AnySync};


// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
// pub struct ErrorData {
//     pub message: String,
//     pub error : Option<String>,
// }

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub error : Option<String>,
    pub logs : Vec<String>,
}

impl ExecutionResponse {
    pub fn new(error: Option<String>, logs: Vec<String>) -> Self {
        ExecutionResponse { error, logs }
    }
}

#[async_trait]
pub trait EmulatorInterface : AnySync
{
    async fn lookup(&self, pubkey: &Pubkey) -> Result<Option<Arc<AccountDataReference>>>;
    async fn execute(
        &self,
        authority : &Pubkey,
        instruction : &instruction::Instruction,
    ) -> Result<ExecutionResponse>;

    /// funds account key from Pubkey::default() account.  If account 'key' is not present, creates
    /// and funds this account.  This fundtion requires presense of Pubkey::default() (SystemProgram) account
    /// that is sufficiently funded.
    /// Please not that in Unit Tests, authority account is automatically funded, i.e. unit tests do not
    /// require presense of SystemProgram account.
    async fn fund(&self, key: &Pubkey, owner: &Pubkey, lamports: u64) -> Result<()>;

    // async fn list(&self) -> Result<Vec<AccountDescriptor>>;
}

downcast_sync!(dyn EmulatorInterface);
