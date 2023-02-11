use wasm_bindgen::prelude::*;
use crate::prelude::Pubkey;
use crate::result::Result;
pub use workflow_wasm::options::OptionsTrait;
use solana_program::instruction::AccountMeta as SolanaAccountMeta;
use js_sys::Object;
use crate::wasm::solana_sys::pubkey_to_jsvalue;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type AccountMeta;
}

impl OptionsTrait for AccountMeta{}


impl AccountMeta{
    /// Set writable
    pub fn is_writable(self, is_writable: bool)->Self{
        self.set("isWritable", JsValue::from(is_writable))
    }

    /// Set signer
    pub fn is_signer(self, is_signer:bool)->Self{
        self.set("isSigner", JsValue::from(is_signer))
    }

    /// Set pubkey
    pub fn pubkey(self, pubkey: &Pubkey)->Result<Self>{
        Ok(self.set("pubkey", pubkey_to_jsvalue(pubkey)?))
    }
}
impl TryFrom<&SolanaAccountMeta> for AccountMeta{
    type Error = crate::error::Error;
    fn try_from(account: &SolanaAccountMeta) -> Result<Self> {
        Ok(AccountMeta::new()
            .is_signer(account.is_signer)
            .is_writable(account.is_writable)
            .pubkey(&account.pubkey)?)
    }
}