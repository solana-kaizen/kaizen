//!
//! Transport interface configuration
//!

use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportMode {
    Inproc,
    Emulator,
    Validator,
}

impl TransportMode {
    pub fn is_emulator(&self) -> bool {
        !matches!(self, TransportMode::Validator)
    }
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub root: Pubkey,
    pub timeout: Duration,
    pub confirm_transaction_initial_timeout: Duration,
    pub retries: usize,
}

impl TransportConfig {
    pub fn new(
        root: Pubkey,
        timeout: Duration,
        confirm_transaction_initial_timeout: Duration,
        retries: usize,
    ) -> TransportConfig {
        TransportConfig {
            root,
            timeout,
            confirm_transaction_initial_timeout,
            retries,
        }
    }

    pub fn default_with_root(root: Pubkey) -> TransportConfig {
        TransportConfig {
            root,
            ..Default::default()
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        TransportConfig {
            root: Pubkey::default(),
            timeout: Duration::from_secs(60u64),
            confirm_transaction_initial_timeout: Duration::from_secs(5u64),
            retries: 2,
        }
    }
}
