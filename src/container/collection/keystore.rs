use super::pubkey::OrderedCollectionInterface;

pub type KeystoreCollection<'info,'refs> = OrderedCollectionInterface<'info,'refs,TsPubkey>;

use std::cmp::Ordering;

use solana_program::pubkey::Pubkey;
use kaizen::time::Instant;
// use kaizen::result::Result;

#[derive(Clone, Copy)]
#[repr(packed)]
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

// ~

impl Ord for TsPubkey {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.ts, &self.key).cmp(&(other.ts, &other.key))
    }
}

impl PartialOrd for TsPubkey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TsPubkey {
    fn eq(&self, other: &Self) -> bool {
        (self.ts, &self.key) == (other.ts, &other.key)
    }
}

impl Eq for TsPubkey { }