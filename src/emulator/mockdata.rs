//!
//! Mock data (mock authority and program_id) for testing.
//!
use crate::utils::generate_random_pubkey;
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct InProcMockData {
    pub authority: Pubkey,
    pub program_id: Pubkey,
}

impl InProcMockData {
    pub fn new(authority: &Pubkey, program_id: &Pubkey) -> Self {
        InProcMockData {
            authority: *authority,
            program_id: *program_id,
        }
    }
}

impl Default for InProcMockData {
    fn default() -> Self {
        InProcMockData {
            authority: generate_random_pubkey(),
            program_id: generate_random_pubkey(),
        }
    }
}
