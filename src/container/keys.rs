use solana_program::pubkey::Pubkey;
use workflow_allocator::time::Instant;
// use workflow_allocator::result::Result;

#[derive(Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct TsPubkey {
    ts : u64,
    key : Pubkey,
}

impl From<(u64, Pubkey)> for TsPubkey {
    fn from((ts, key): (u64, Pubkey)) -> Self {
        TsPubkey { ts, key }
    }
}

impl From<(Instant, &Pubkey)> for TsPubkey {
    fn from((ts, key): (Instant, &Pubkey)) -> Self {
        TsPubkey { ts : ts.0, key : *key }
    }
}

