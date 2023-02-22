//use crate::result::Result;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
//use workflow_wasm::abi::ref_from_abi;
use serde_wasm_bindgen::from_value;

/*
pub trait PubkeyExt {
    fn try_from(value: &JsValue) -> std::result::Result<Pubkey, JsValue>;
}

impl PubkeyExt for Pubkey {
    fn try_from(value: &JsValue) -> std::result::Result<Pubkey, JsValue> {
        if value.is_string() {
            Ok(Pubkey::from_str(&value.as_string().unwrap())
                .map_err(|_| JsValue::from("Invalid pubkey"))?)
        } else {
            Ok(ref_from_abi!(Pubkey, value)?)
        }
    }
}
*/

pub fn pubkey_from_value(pubkey: JsValue) -> std::result::Result<Pubkey, JsValue> {
    let pubkey = if let Some(pubkey_str) = pubkey.as_string() {
        Pubkey::from_str(&pubkey_str).map_err(|_| JsValue::from("Invalid pubkey"))?
    } else {
        from_value(pubkey)?
    };

    Ok(pubkey)
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
