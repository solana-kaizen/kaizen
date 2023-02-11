use wasm_bindgen::prelude::*;
//use crate::prelude::Pubkey;
//use crate::result::Result;
pub use workflow_wasm::options::OptionsTrait;
use crate::wasm::solana_sys::*;

use js_sys::{
    Object
};


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WalletAdapter;

    #[wasm_bindgen(getter, method, js_name="publicKey")]
    /// get pubKey
    ///
    pub fn pubkey(this: &WalletAdapter)->JsValue;

    #[wasm_bindgen(method, js_name="signTransaction")]
    /// sign transaction
    ///
    pub async fn sign_transaction_impl(this: &WalletAdapter, tx: Transaction)->JsValue;
}


impl WalletAdapter{
    pub async fn sign_transaction(&self, tx: &Transaction)->Result<JsValue>{
        Ok(self.sign_transaction_impl(tx.clone()).await)
    }
}
