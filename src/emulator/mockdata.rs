use solana_program::pubkey::Pubkey;
use crate::generate_random_pubkey;

#[derive(Clone, Debug)]
pub struct InProcMockData {
    pub authority : Pubkey,
    pub program_id: Pubkey,
}

impl InProcMockData {
    pub fn new(
        authority : &Pubkey,
        program_id : &Pubkey,
    ) -> Self {
        InProcMockData {
            authority : authority.clone(),
            program_id : program_id.clone(),
        }
    }
}

impl Default for InProcMockData {
    fn default() -> Self {
        InProcMockData {
            authority : generate_random_pubkey(),
            program_id: generate_random_pubkey(),
        }
    }
}