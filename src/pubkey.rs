use crate::result::Result;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use workflow_wasm::abi::ref_from_abi;

pub trait PubkeyExt {
    fn try_from(value: &JsValue) -> Result<Pubkey>;
}

impl PubkeyExt for Pubkey {
    fn try_from(value: &JsValue) -> Result<Pubkey> {
        if value.is_string() {
            Ok(Pubkey::from_str(&value.as_string().unwrap())?)
        } else {
            Ok(ref_from_abi!(Pubkey, value)?)
        }
    }
}

/// Generates a [`Pubkey`] filled with random bytes (used explicitly in unit tests)
#[cfg(not(target_os = "solana"))]
pub fn generate_random_pubkey() -> Pubkey {
    // Pubkey::new(&rand::random::<[u8; 32]>())
    Pubkey::new_from_array(rand::random::<[u8; 32]>())
}

#[cfg(target_os = "solana")]
pub fn generate_random_pubkey() -> Pubkey {
    Pubkey::new_unique()
}
