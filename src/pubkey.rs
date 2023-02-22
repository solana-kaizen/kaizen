//use crate::result::Result;
use serde_wasm_bindgen::from_value;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use workflow_wasm::abi::ref_from_abi;

pub trait PubkeyExt {
    fn from_value(value: &JsValue) -> std::result::Result<Pubkey, JsValue>;
}

impl PubkeyExt for Pubkey {
    fn from_value(value: &JsValue) -> std::result::Result<Pubkey, JsValue> {
        if value.is_string() {
            Ok(Pubkey::from_str(&value.as_string().unwrap())
                .map_err(|_| JsValue::from("Invalid pubkey"))?)
        } else if value.is_array() {
            Ok(from_value(value.into())?)
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
