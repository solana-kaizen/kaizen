use solana_program::pubkey::Pubkey;
use crate::generate_random_pubkey;

#[derive(Clone, Debug)]
pub struct InProcMockData {
    pub authority : Pubkey,
    pub identity: Pubkey,
    pub program_id: Pubkey,
}

impl InProcMockData {
    pub fn new() -> Self {
        InProcMockData {
            authority : generate_random_pubkey(),
            identity: generate_random_pubkey(),
            program_id: generate_random_pubkey(),
        }
    }
}
