use wasm_bindgen::prelude::*;
use crate::prelude::Pubkey;
use crate::result::Result;
pub use workflow_wasm::options::OptionsTrait;
use js_sys::{Array, Uint8Array};

mod connection;
mod tx_instruction;
mod tx_instraction_config;
mod tx;
mod account_meta;
mod wallet_adapter;
pub use tx_instruction::*;
pub use tx_instraction_config::*;
pub use tx::*;
pub use account_meta::*;
pub use connection::*;
pub use wallet_adapter::*;


pub fn pubkey_to_jsvalue(pubkey: &Pubkey) -> Result<JsValue> {
    let pubkey_bytes = pubkey.to_bytes();
    let u8arr = Uint8Array::from(&pubkey_bytes[..]);
    let pkargs = Array::new_with_length(1);
    pkargs.set(0 as u32, u8arr.into());
    let ctor = js_sys::Reflect::get(&super::solana()?, &JsValue::from("PublicKey"))?;
    let pk_jsv = js_sys::Reflect::construct(&ctor.into(), &pkargs)?;
    Ok(pk_jsv)
}

