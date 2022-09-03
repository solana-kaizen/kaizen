use solana_program::pubkey::Pubkey;
use crate::generate_random_pubkey;

#[derive(Clone, Debug)]
pub struct InProcMockData {
    pub authority : Pubkey,
    // pub identity: Pubkey,
    pub program_id: Pubkey,
}

impl InProcMockData {
    pub fn new(
        authority : &Pubkey,
        // identity : &Pubkey,
        program_id : &Pubkey,
    ) -> Self {
        InProcMockData {
            authority : authority.clone(),
            // identity : identity.clone(),
            program_id : program_id.clone(),
        }
    }
}

impl Default for InProcMockData {
    fn default() -> Self {
        InProcMockData {
            authority : generate_random_pubkey(),
            // identity: generate_random_pubkey(),
            program_id: generate_random_pubkey(),
        }
    }
}